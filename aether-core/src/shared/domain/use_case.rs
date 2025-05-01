use std::error::Error;

use async_trait::async_trait;

pub trait UseCase {
    type Input;
    type Output;
    type Error: Error;

    fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}

#[async_trait]
pub trait AsyncUseCase {
    type Output;

    async fn execute(&self) -> Self::Output;
}

#[async_trait]
pub trait AsyncUseCaseWithInput {
    type Input;
    type Output;

    async fn execute(&self, input: Self::Input) -> Self::Output;
}

#[async_trait]
pub trait AsyncUseCaseWithError {
    type Output;
    type Error: Error;

    async fn execute(&self) -> Result<Self::Output, Self::Error>;
}

#[async_trait]
pub trait AsyncUseCaseWithInputAndError {
    type Input;
    type Output;
    type Error: Error;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}
