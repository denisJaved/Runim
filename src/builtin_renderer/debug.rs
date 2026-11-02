#[cfg(feature = "debugging")]
pub struct DebugName(pub String);

#[cfg(not(feature = "debugging"))]
pub struct DebugName();

#[cfg(feature = "debugging")]

impl DebugName {
    pub fn name(&self) -> Option<&str> {
        Some(&self.0)
    }
}

#[cfg(not(feature = "debugging"))]
impl DebugName {
    pub fn name(&self) -> Option<&str> {
        None
    }
}


#[cfg(feature = "debugging")]
#[macro_export]
macro_rules! dbgn {
    ( $($arg:tt)* ) => {
        crate::builtin_renderer::debug::DebugName(String::from(format!($($arg,)*)))
    };
}

#[cfg(not(feature = "debugging"))]
#[macro_export]
macro_rules! dbgn {
    ( $($arg:tt)* ) => {
        crate::builtin_renderer::debug::DebugName()
    };
}
