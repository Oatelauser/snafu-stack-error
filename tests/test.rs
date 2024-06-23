use std::{fs, io};

use snafu::{Location, location, ResultExt, Snafu};

use snafu_stack_error::StackError;
use stack_error_derive::stack_trace_debug;

#[derive(Snafu)]
#[snafu(visibility(pub))]
#[stack_trace_debug]
enum MyError {
    #[snafu(display("this has io error1"))]
    IO {
        #[snafu(source)]
        error: io::Error,
        // #[snafu(implicit)]
        location: Location,
    },
}

#[derive(Snafu)]
#[snafu(visibility(pub))]
#[stack_trace_debug]
enum TestError<const N: usize> {
    #[snafu(display("this has io error2"))]
    Invalid {
        source: MyError,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Unrecognized database option key: {}", key))]
    InvalidDatabaseOption {
        key: String,
        location: Location,
    },
}

type Result<T> = std::result::Result<T, TestError<1>>;

#[test]
fn test() -> Result<()> {
    let path = "config.toml";
    // let r = fs::read(path).context(IOSnafu { location: location!() })
    //     .context(InvalidSnafu);
    let error: TestError<1> = InvalidDatabaseOptionSnafu { key: String::from("Alice"), location: location!() }.build();
    // let a = error.last();
    let a = <TestError<1> as StackError>::last(&error);
    println!("{}", a);
    Ok(())
    // ensure!(false, InvalidDatabaseOptionSnafu { key: String::from("Alice"), location: location!()});
}
