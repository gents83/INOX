#[macro_export]
macro_rules! implement_message {
    ($Type:ident, $Func:ident, $Policy:ident) => {
        impl $crate::Message for $Type {
            #[inline]
            fn compare_and_discard(&self, other: &Self) -> bool {
                Self::$Policy(self, other)
            }
            #[inline]
            fn from_command_parser(command_parser: CommandParser) -> Option<Self>
            where
                Self: Sized,
            {
                Self::$Func(command_parser)
            }
        }
    };
    ($Type:ident, $Policy:ident) => {
        impl $crate::Message for $Type {
            #[inline]
            fn compare_and_discard(&self, other: &Self) -> bool {
                Self::$Policy(self, other)
            }
            #[inline]
            fn from_command_parser(command_parser: CommandParser) -> Option<Self>
            where
                Self: Sized,
            {
                None
            }
        }
    };
    ($Type:ident<$InnerType:ident>, $Func:ident, $Policy:ident) => {
        impl<T> $crate::Message for $Type<T>
        where
            T: $InnerType,
            $Type<T>: 'static,
        {
            #[inline]
            fn compare_and_discard(&self, other: &Self) -> bool {
                Self::$Policy(self, other)
            }
            #[inline]
            fn from_command_parser(command_parser: CommandParser) -> Option<Self>
            where
                Self: Sized,
            {
                Self::$Func(command_parser)
            }
        }
    };
    ($Type:ident<$InnerType:ident>, $Policy:ident) => {
        impl<T> $crate::Message for $Type<T>
        where
            T: $InnerType,
        {
            #[inline]
            fn compare_and_discard(&self, other: &Self) -> bool {
                Self::$Policy(self, other)
            }
            #[inline]
            fn from_command_parser(command_parser: CommandParser) -> Option<Self>
            where
                Self: Sized,
            {
                None
            }
        }
    };
}

#[macro_export]
macro_rules! implement_undoable_message {
    ($Type:ident, $func: ident, $debug_func: ident) => {
        impl $crate::Message for $Type {
            #[inline]
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            #[inline]
            fn as_boxed(&self) -> Box<dyn $crate::Message> {
                Box::new(self.clone())
            }
            #[inline]
            fn redo(&self, events_rw: &$crate::MessageBox) {
                let mut events = events_rw.write().unwrap();
                events.send(self.as_boxed()).ok();
            }
            #[inline]
            fn undo(&self, events_rw: &$crate::MessageBox) {
                let mut events = events_rw.write().unwrap();
                let event_to_send = $func(self);
                events.send(event_to_send.as_boxed()).ok();
            }
            #[inline]
            fn get_debug_info(&self) -> String {
                $debug_func(self)
            }
            #[inline]
            fn send_to(self: Box<Self>, message_hub: &mut $crate::MessageHub) {
                let msg = Box::leak(self);
                message_hub.send_event(msg);
            }
        }
    };
}
