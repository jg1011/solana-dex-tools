use anyhow::Result; 

/// quick alias for anyhow::Result, avoids confusion with std::result::Result
pub type AnyResult<T> = Result<T>;