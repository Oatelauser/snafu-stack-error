# 错误堆栈最佳实践

## 目录

- [范式规则](#范式规则)
- [范式定义](#范式定义)
- [错误的使用](#错误的使用)

#### 范式规则

错误堆栈的最佳实践范式规则如下

1. 每一个错误都必须定义`#[snafu(display)]`的错误信息
2. 每一个内部错误必须实现ErrorExt，用于判断哪些错误对外展示什么信息
3. 一个模块内部使用一个枚举类定义错误，达到内部统一的错误类型
4. 模块内部对接外部错误（非当前模块的错误），必须定义错误传递者代理接受外部错误
5. 模块之间的对接，也遵循规则3，需要定义错误传递者接受其他模块的错误，这个错误的命名一般是以模块名命名

***

#### 范式定义

以`GreptimeDB`项目代码为例，它的错误范式定义

- [错误的命名规范](https://www.wolai.com/wnHMUbQEHcvyygUYaX32kG "错误的命名规范")
- 每个模块存在`error`文件用于定义当前模块用到的所有错误

```rust
#[derive(Snafu)]
#[snafu(visibility(pub))]
#[stack_trace_debug]
pub enum Error {

    #[snafu(display("Invalid utf-8 value"))]
    InvalidUtf8Value {
        #[snafu(source)]
        error: FromUtf8Error,     // 关联外部错误 
        location: Location,
    },

    #[snafu(display("Error accessing catalog"))]
    CatalogError {
        source: catalog::error::Error,   // 关联内部错误 
        #[snafu(implicit)]
        location: Location,
    },
    
}

impl ErrorExt for Error {
    fn status_code(&self) -> StatusCode {
        use Error::*;
        match self {
            InvalidUtf8Value { .. } | InvalidFlushArgument { .. } => StatusCode::InvalidArguments,
            CatalogError { .. } => StatusCode::Internal,
            UnsupportedDataType { .. } => StatusCode::Unsupported,
            CollectRecordbatch { .. } => StatusCode::EngineExecuteQuery,
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub type Result<T> = std::result::Result<T, Error>;   // 定义当前错误的 Result 类，方便代码的使用
```

1. `#[derive(Snafu)]`：使用[snafu](https://www.wolai.com/sqiv7yJfpjxzuoH22jw3vy "snafu")过程宏定义错误对象
2. stack\_trace\_debug：自动实现StackError和`Display`特征
3. `#[snafu(display)]`：每个枚举值必须添加异常信息，且不能为空字符串
4. 内部错误：定义的错误对象使用`source`字段关联内部错误
5. 外部错误：定义的错误对象使用`error`字段关联外部错误，且必须标识`#[snafu(source)]`，因为snafu不识别`error`字段
6. `Location`：对于代码位置`Location`的定义，不推荐使用[#\[snafu(implicit)\]](https://www.wolai.com/3F67VYg8inPT8JdLwfmZvD "#\[snafu(implicit)]")，而是显示调用location!
7. 内部错误需要实现ErrorExt特征
8. 每个error文件都会定义自己的`Result`类型

***

#### 错误的使用

以`GreptimeDB`对于错误的处理为例，该项目存在多个模块，例如auth模块和servers模块的交互为例

1. auth模块提供`check_permission`方法暴露它自身的`Error`错误
   ```rust
   // error.rs
   #[derive(Snafu)]
   #[snafu(visibility(pub))]
   #[stack_trace_debug]
   pub enum Error {     // auth模块定义的错误 
       #[snafu(display("User is not authorized to perform this action"))]
       PermissionDenied {
           #[snafu(implicit)]
           location: Location,
       },
       ... ...
   }


   // permission.rs
   impl PermissionChecker for Option<&PermissionCheckerRef> {
       fn check_permission(
           &self,
           user_info: Option<UserInfoRef>,
           req: PermissionReq,
       ) -> Result<PermissionResp> {
           match self {
               Some(checker) => match checker.check_permission(user_info, req) {
                   Ok(PermissionResp::Reject) => PermissionDeniedSnafu.fail(),   // 如果校验被拒绝，则直接抛出 PermissionDenied 内部错误 
                   Ok(PermissionResp::Allow) => Ok(PermissionResp::Allow),
                   Err(e) => Err(e),  // 如果是校验方法出现错误，则直接抛出
               },
               None => Ok(PermissionResp::Allow),
           }
       }
   }
   ```
2. servers模块提供`metrics`方法，内部调用`auth::check_permission`并做了错误转换
   ```rust
   #[derive(Snafu)]
   #[snafu(visibility(pub))]
   #[stack_trace_debug]
   pub enum Error {   // servers模块定义的错误 

       #[snafu(display("Failed to get user info"))]
       Auth {
           #[snafu(implicit)]
           location: Location,
           source: auth::error::Error,
       },
       
       #[snafu(display("Execute gRPC query error"))]
       ExecuteGrpcQuery {
           #[snafu(implicit)]
           location: Location,
           source: BoxedError,
       },
       ... ...
   }

   #[async_trait]
   impl OpenTelemetryProtocolHandler for Instance {

       #[tracing::instrument(skip_all)]
       async fn metrics(
           &self,
           request: ExportMetricsServiceRequest,
           ctx: QueryContextRef,
       ) -> ServerResult<Output> {
           self.plugins
               .get::<PermissionCheckerRef>()
               .as_ref()
               .check_permission(ctx.current_user(), PermissionReq::Otlp)   // 调用 auth::check_permission 方法 
               .context(AuthSnafu)?;   // 将auth模块的错误转换为当前模块的错误，也就是错误传递者 

           let interceptor_ref = self
               .plugins
               .get::<OpenTelemetryProtocolInterceptorRef<servers::error::Error>>();
           interceptor_ref.pre_execute(ctx.clone())?;   // 可能产生当前模块的错误

           let (requests, rows) = otlp::metrics::to_grpc_insert_requests(request)?;     // 可能产生当前模块的错误
           OTLP_METRICS_ROWS.inc_by(rows as u64);

           self.handle_row_inserts(requests, ctx)
               .await
               .map_err(BoxedError::new)
               .context(error::ExecuteGrpcQuerySnafu)     // frontend模块的错误转换为当前模块的错误 
       }

   }
   ```
3. 最外层的代码处理这些范式错误，根据业务需求，使用实现的ErrorExt、StackError特征处理
   ```rust
   fn test() -> Result<()> {
       let path = "config.toml";
       // let r = fs::read(path).context(IOSnafu { location: location!() })
       //     .context(InvalidSnafu);
       let error: TestError = InvalidDatabaseOptionSnafu { key: String::from("Alice"), location: location!() }.build();
       // let a = error.last();
       let a = <TestError as StackError>::last(&error);
       println!("{}", a);
       Ok(())
       // ensure!(false, InvalidDatabaseOptionSnafu { key: String::from("Alice"), location: location!()});
   }
   ```
