#[macro_export]
macro_rules! implement_message {
    ($Type:ident, $Func:ident) => {
        impl $crate::Message for $Type {
            #[inline]
            fn send(self: Box<Self>, message_hub: &mut $crate::MessageHub) {
                let msg = unsafe { *self };
                message_hub.send_event(msg);
            }
        }
        impl $crate::MessageFromString for $Type {
            #[inline]
            fn from_command_parser(command_parser: CommandParser) -> Option<Self>
            where
                Self: Sized,
            {
                Self::$Func(command_parser)
            }
        }
    };
    ($Type:ident, $RestrictedType:ident, $Func:ident) => {
        impl $crate::Message for $Type {
            #[inline]
            fn send(self: Box<Self>, message_hub: &mut $crate::MessageHub) {
                let msg = unsafe { *self };
                message_hub.send_event(msg);
            }
        }
        impl $crate::MessageFromString for $RestrictedType {
            #[inline]
            fn from_command_parser(command_parser: CommandParser) -> Option<Self>
            where
                Self: Sized,
            {
                Self::$Func(command_parser)
            }
        }
    };
    ($Type:ident) => {
        impl $crate::Message for $Type {
            #[inline]
            fn send(self: Box<Self>, message_hub: &mut $crate::MessageHub) {
                let msg = unsafe { *self };
                message_hub.send_event(msg);
            }
        }
    };
    ($Type:ident<$InnerType:ident>, $Func:ident) => {
        impl<T> $crate::Message for $Type<T>
        where
            T: $InnerType,
        {
            #[inline]
            fn send(self: Box<Self>, message_hub: &mut $crate::MessageHub) {
                let msg = unsafe { *self };
                message_hub.send_event(msg);
            }
        }
        impl<T> $crate::MessageFromString for $Type<T>
        where
            T: $InnerType,
        {
            #[inline]
            fn from_command_parser(command_parser: CommandParser) -> Option<Self>
            where
                Self: Sized,
            {
                Self::$Func(command_parser)
            }
        }
    };
    ($Type:ident<$InnerType:ident>, $SameType:ident<$RestrictedType:ident>, $Func:ident) => {
        impl<T> $crate::Message for $Type<T>
        where
            T: $InnerType,
        {
            #[inline]
            fn send(self: Box<Self>, message_hub: &mut $crate::MessageHub) {
                let msg = unsafe { *self };
                message_hub.send_event(msg);
            }
        }
        impl<T> $crate::MessageFromString for $Type<T>
        where
            T: $RestrictedType,
        {
            #[inline]
            fn from_command_parser(command_parser: CommandParser) -> Option<Self>
            where
                Self: Sized,
            {
                Self::$Func(command_parser)
            }
        }
    };
    ($Type:ident<$InnerType:ident>) => {
        impl<T> $crate::Message for $Type<T>
        where
            T: $InnerType,
        {
            #[inline]
            fn send(self: Box<Self>, message_hub: &mut $crate::MessageHub) {
                let msg = unsafe { *self };
                message_hub.send_event(msg);
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
