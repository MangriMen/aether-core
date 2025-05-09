use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;

use crate::{features::java::ports::JreProvider, shared::domain::AsyncUseCaseWithInputAndError};

pub struct InstallJreUseCase<JP: JreProvider> {
    provider: Arc<JP>,
}

impl<JP: JreProvider> InstallJreUseCase<JP> {
    pub fn new(provider: Arc<JP>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl<JP: JreProvider> AsyncUseCaseWithInputAndError for InstallJreUseCase<JP> {
    type Input = (u32, PathBuf);
    type Output = PathBuf;
    type Error = crate::Error;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        self.provider.install(input.0, &input.1).await
    }
}
