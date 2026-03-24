#[macro_export]
macro_rules! msg_adapter_impl {
    ($message:ident, $from:ident, $name:ident) => {
        impl From<$message> for $from {
            fn from(value: $message) -> Self {
                $from::$name(value)
            }
        }

        impl $crate::app::FromOrPanic<$from> for $message {
            fn from_or_panic(value: $from) -> Self {
                match value {
                    $from::$name(message) => message,
                    _ => unreachable!(),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! name_of_trait {
    ($t:ident) => {{
        let _: &dyn $t;
        stringify!($t)
    }};
}
