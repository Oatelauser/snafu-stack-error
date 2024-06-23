
use async_trait::async_trait;
use snafu::location;
use tokio_test::block_on;
use snafu::{Location, Snafu};

use crate::error::{Error, InvalidDatabaseOptionSnafu};

pub mod error {
    use snafu::{Location, Snafu};

    use stack_error_derive::stack_trace_debug;

    #[derive(Snafu)]
    #[snafu(visibility(pub))]
    #[stack_trace_debug]
    pub enum Error {
        #[snafu(display("Unrecognized"))]
        InvalidDatabaseOption {
            location: Location,
        },
    }
}

#[async_trait]
trait MyAsyncTrait {
    async fn async_method(&self) -> Result<(), Error>;
}

struct MyStruct(bool);

#[async_trait]
impl MyAsyncTrait for MyStruct {
    async fn async_method(&self) -> Result<(), Error> {
        if self.0 {
            Ok(())
        } else {
            InvalidDatabaseOptionSnafu { location: location!() }.fail()
        }
    }
}

#[test]
fn test() {
    let s = MyStruct(false);
    let result = block_on(s.async_method());
    result.unwrap();
}