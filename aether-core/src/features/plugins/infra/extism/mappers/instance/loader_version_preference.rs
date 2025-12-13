use crate::features::{
    minecraft::LoaderVersionPreference, plugins::v0::LoaderVersionPreferenceDto,
};

impl From<LoaderVersionPreferenceDto> for LoaderVersionPreference {
    fn from(value: LoaderVersionPreferenceDto) -> Self {
        match value {
            LoaderVersionPreferenceDto::Latest => Self::Latest,
            LoaderVersionPreferenceDto::Stable => Self::Stable,
            LoaderVersionPreferenceDto::Exact(version) => Self::Exact(version),
        }
    }
}
