use crate::max_array::MaxArray;


macro_rules! implement_horner {
    ($name:ident, $type:ty, $zero:expr, $one:expr, $two:expr $(, $max_bits:expr)?) => {
        fn $name(&self, lowest_bit_index: usize, bit_count: usize) -> Option<$type> {
            $(assert!(bit_count <= $max_bits);)?
            let mut value = $zero;
            let mut factor = $one;
            for i in lowest_bit_index..lowest_bit_index+bit_count {
                if self.bit_is_set(i)? {
                    value += factor;
                }
                if i < lowest_bit_index+bit_count-1 {
                    factor *= $two;
                }
            }
            Some(value)
        }
    };
}


trait BitTwiddling {
    /// Returns whether the bit at the given index is set.
    fn bit_is_set(&self, bit_index: usize) -> Option<bool>;

    fn bool_from_bits(&self, bit_index: usize, bit_count: usize) -> Option<bool> {
        assert!(bit_count <= 1);
        if bit_count == 1 {
            self.bit_is_set(bit_index)
        } else {
            None
        }
    }

    implement_horner!(u8_from_bits, u8, 0, 1, 2, 8);
    implement_horner!(u16_from_bits, u16, 0, 1, 2, 16);
    implement_horner!(u32_from_bits, u32, 0, 1, 2, 32);
    implement_horner!(u64_from_bits, u64, 0, 1, 2, 64);
    implement_horner!(f64_from_bits, f64, 0.0, 1.0, 2.0);
}
impl<'a> BitTwiddling for &'a [u8] {
    // this implementation assumes that bytes are in little-endian order,
    // i.e. from least to most significant;
    // the bits within a byte are also considered to be little-endian
    // (index 0 is LSb of self[0], index 7 is MSb of self[0], index 8 is LSb of self[1], etc.)

    fn bit_is_set(&self, bit_index: usize) -> Option<bool> {
        let byte_index = bit_index / 8;
        let bit_in_byte_index = bit_index % 8;
        if byte_index >= self.len() {
            return None;
        }

        Some(self[byte_index] & (1 << bit_in_byte_index) != 0)
    }
}


fn range_scale(mut value: f64, min_range: f64, max_range: f64, min_scale: f64, max_scale: f64) -> f64 {
    let bottom_range = min_range.min(max_range);
    let top_range = min_range.max(max_range);
    if value < bottom_range {
        value = bottom_range;
    }
    if value > top_range {
        value = top_range;
    }

    let value_zeroed = value - min_range;
    let value_zeroed_scaled = value_zeroed * (max_scale - min_scale) / (max_range - min_range);
    let value_scaled = value_zeroed_scaled + min_scale;
    value_scaled
}


/// RPS Telegram (F6)
#[allow(non_snake_case)]
pub mod rorgF6 {
    #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
    /// Switch Buttons (F6-01)
    pub mod func01 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Push Button (F6-01-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Push button value.
            pub fn get_push_button_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(3, 1)
            }
            /// Get the Push button value.
            pub fn get_push_button(&self) -> Option<Type01PropPushButton> {
                let raw_value = self.get_push_button_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("push_button", &self.get_push_button())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropPushButton {
            Released = false,
            Pressed = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func01<'b> {
        Type01(func01::Type01<'b>),
    }
    impl<'b> Func01<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func01::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Rocker Switch, 2 Rocker (F6-02)
    pub mod func02 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Light and Blind Control - Application Style 1 (F6-02-01), case 0
        #[derive(Clone, Copy)]
        pub struct Type01Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Rocker 1st action value.
            pub fn get_rocker_1st_action_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 3)
            }
            /// Get the Rocker 1st action value.
            pub fn get_rocker_1st_action(&self) -> Option<Type01Case0PropRocker1stAction> {
                let raw_value = self.get_rocker_1st_action_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy Bow value.
            pub fn get_energy_bow_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(3, 1)
            }
            /// Get the Energy Bow value.
            pub fn get_energy_bow(&self) -> Option<Type01Case0PropEnergyBow> {
                let raw_value = self.get_energy_bow_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Rocker 2nd action value.
            pub fn get_rocker_2nd_action_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 3)
            }
            /// Get the Rocker 2nd action value.
            pub fn get_rocker_2nd_action(&self) -> Option<Type01Case0PropRocker2ndAction> {
                let raw_value = self.get_rocker_2nd_action_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw 2nd Action value.
            pub fn get_2nd_action_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(7, 1)
            }
            /// Get the 2nd Action value.
            pub fn get_2nd_action(&self) -> Option<Type01Case0Prop2ndAction> {
                let raw_value = self.get_2nd_action_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01Case0")
                    .field("rocker_1st_action", &self.get_rocker_1st_action())
                    .field("energy_bow", &self.get_energy_bow())
                    .field("rocker_2nd_action", &self.get_rocker_2nd_action())
                    .field("2nd_action", &self.get_2nd_action())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case0PropRocker1stAction {
            ButtonAi = 0,
            ButtonA0 = 1,
            ButtonBi = 2,
            ButtonB0 = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case0PropEnergyBow {
            Released = false,
            Pressed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case0PropRocker2ndAction {
            ButtonAi = 0,
            ButtonA0 = 1,
            ButtonBi = 2,
            ButtonB0 = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case0Prop2ndAction {
            No2ndAction = false,
            _2ndActionValid = true,
            _Other(bool),
        }
        /// Light and Blind Control - Application Style 1 (F6-02-01), case 1
        #[derive(Clone, Copy)]
        pub struct Type01Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Number of buttons pressed simultaneously (other bit combinations are not valid) value.
            pub fn get_number_of_buttons_pressed_simultaneously_other_bit_combinations_are_not_valid_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 3)
            }
            /// Get the Number of buttons pressed simultaneously (other bit combinations are not valid) value.
            pub fn get_number_of_buttons_pressed_simultaneously_other_bit_combinations_are_not_valid(&self) -> Option<Type01Case1PropNumberOfButtonsPressedSimultaneouslyOtherBitCombinationsAreNotValid> {
                let raw_value = self.get_number_of_buttons_pressed_simultaneously_other_bit_combinations_are_not_valid_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy Bow value.
            pub fn get_energy_bow_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(3, 1)
            }
            /// Get the Energy Bow value.
            pub fn get_energy_bow(&self) -> Option<Type01Case1PropEnergyBow> {
                let raw_value = self.get_energy_bow_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01Case1")
                    .field("number_of_buttons_pressed_simultaneously_other_bit_combinations_are_not_valid", &self.get_number_of_buttons_pressed_simultaneously_other_bit_combinations_are_not_valid())
                    .field("energy_bow", &self.get_energy_bow())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case1PropNumberOfButtonsPressedSimultaneouslyOtherBitCombinationsAreNotValid {
            NoButton = 0,
            _3Or4Buttons = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropEnergyBow {
            Released = false,
            Pressed = true,
            _Other(bool),
        }
        /// Light and Blind Control - Application Style 2 (F6-02-02), case 0
        #[derive(Clone, Copy)]
        pub struct Type02Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Rocker 1st action value.
            pub fn get_rocker_1st_action_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 3)
            }
            /// Get the Rocker 1st action value.
            pub fn get_rocker_1st_action(&self) -> Option<Type02Case0PropRocker1stAction> {
                let raw_value = self.get_rocker_1st_action_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy Bow value.
            pub fn get_energy_bow_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(3, 1)
            }
            /// Get the Energy Bow value.
            pub fn get_energy_bow(&self) -> Option<Type02Case0PropEnergyBow> {
                let raw_value = self.get_energy_bow_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Rocker 2nd action value.
            pub fn get_rocker_2nd_action_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 3)
            }
            /// Get the Rocker 2nd action value.
            pub fn get_rocker_2nd_action(&self) -> Option<Type02Case0PropRocker2ndAction> {
                let raw_value = self.get_rocker_2nd_action_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw 2nd Action value.
            pub fn get_2nd_action_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(7, 1)
            }
            /// Get the 2nd Action value.
            pub fn get_2nd_action(&self) -> Option<Type02Case0Prop2ndAction> {
                let raw_value = self.get_2nd_action_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02Case0")
                    .field("rocker_1st_action", &self.get_rocker_1st_action())
                    .field("energy_bow", &self.get_energy_bow())
                    .field("rocker_2nd_action", &self.get_rocker_2nd_action())
                    .field("2nd_action", &self.get_2nd_action())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type02Case0PropRocker1stAction {
            ButtonAi = 0,
            ButtonA0 = 1,
            ButtonBi = 2,
            ButtonB0 = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02Case0PropEnergyBow {
            Released = false,
            Pressed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type02Case0PropRocker2ndAction {
            ButtonAi = 0,
            ButtonA0 = 1,
            ButtonBi = 2,
            ButtonB0 = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02Case0Prop2ndAction {
            No2ndAction = false,
            _2ndActionValid = true,
            _Other(bool),
        }
        /// Light and Blind Control - Application Style 2 (F6-02-02), case 1
        #[derive(Clone, Copy)]
        pub struct Type02Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Number of buttons pressed simultaneously (other bit combinations are not valid) value.
            pub fn get_number_of_buttons_pressed_simultaneously_other_bit_combinations_are_not_valid_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 3)
            }
            /// Get the Number of buttons pressed simultaneously (other bit combinations are not valid) value.
            pub fn get_number_of_buttons_pressed_simultaneously_other_bit_combinations_are_not_valid(&self) -> Option<Type02Case1PropNumberOfButtonsPressedSimultaneouslyOtherBitCombinationsAreNotValid> {
                let raw_value = self.get_number_of_buttons_pressed_simultaneously_other_bit_combinations_are_not_valid_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy Bow value.
            pub fn get_energy_bow_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(3, 1)
            }
            /// Get the Energy Bow value.
            pub fn get_energy_bow(&self) -> Option<Type02Case1PropEnergyBow> {
                let raw_value = self.get_energy_bow_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02Case1")
                    .field("number_of_buttons_pressed_simultaneously_other_bit_combinations_are_not_valid", &self.get_number_of_buttons_pressed_simultaneously_other_bit_combinations_are_not_valid())
                    .field("energy_bow", &self.get_energy_bow())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type02Case1PropNumberOfButtonsPressedSimultaneouslyOtherBitCombinationsAreNotValid {
            NoButton = 0,
            _3Or4Buttons = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02Case1PropEnergyBow {
            Released = false,
            Pressed = true,
            _Other(bool),
        }
        /// Light Control - Application Style 1 (F6-02-03)
        #[derive(Clone, Copy)]
        pub struct Type03<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type03<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Rocker action value.
            pub fn get_rocker_action_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Rocker action value.
            pub fn get_rocker_action(&self) -> Option<Type03PropRockerAction> {
                let raw_value = self.get_rocker_action_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type03<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type03")
                    .field("rocker_action", &self.get_rocker_action())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type03PropRockerAction {
            ButtonA0 = 48,
            ButtonA1 = 16,
            ButtonB0 = 112,
            ButtonB1 = 80,
            _Other(u8),
        }
        /// Light and blind control ERP2 (F6-02-04)
        #[derive(Clone, Copy)]
        pub struct Type04<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type04<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Energy Bow value.
            pub fn get_energy_bow_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(0, 1)
            }
            /// Get the Energy Bow value.
            pub fn get_energy_bow(&self) -> Option<Type04PropEnergyBow> {
                let raw_value = self.get_energy_bow_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Button coding value.
            pub fn get_button_coding_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(1, 1)
            }
            /// Get the Button coding value.
            pub fn get_button_coding(&self) -> Option<Type04PropButtonCoding> {
                let raw_value = self.get_button_coding_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw BI value.
            pub fn get_bi_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(4, 1)
            }
            /// Get the BI value.
            pub fn get_bi(&self) -> Option<Type04PropBi> {
                let raw_value = self.get_bi_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw B0 value.
            pub fn get_b0_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(5, 1)
            }
            /// Get the B0 value.
            pub fn get_b0(&self) -> Option<Type04PropB0> {
                let raw_value = self.get_b0_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw AI value.
            pub fn get_ai_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(6, 1)
            }
            /// Get the AI value.
            pub fn get_ai(&self) -> Option<Type04PropAi> {
                let raw_value = self.get_ai_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw A0 value.
            pub fn get_a0_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(7, 1)
            }
            /// Get the A0 value.
            pub fn get_a0(&self) -> Option<Type04PropA0> {
                let raw_value = self.get_a0_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type04<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type04")
                    .field("energy_bow", &self.get_energy_bow())
                    .field("button_coding", &self.get_button_coding())
                    .field("bi", &self.get_bi())
                    .field("b0", &self.get_b0())
                    .field("ai", &self.get_ai())
                    .field("a0", &self.get_a0())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropEnergyBow {
            Released = false,
            Pressed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropButtonCoding {
            Button = false,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropBi {
            NotPressed = false,
            Pressed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropB0 {
            NotPressed = false,
            Pressed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropAi {
            NotPressed = false,
            Pressed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropA0 {
            NotPressed = false,
            Pressed = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func02<'b> {
        Type01Case0(func02::Type01Case0<'b>),
        Type01Case1(func02::Type01Case1<'b>),
        Type02Case0(func02::Type02Case0<'b>),
        Type02Case1(func02::Type02Case1<'b>),
        Type03(func02::Type03<'b>),
        Type04(func02::Type04<'b>),
    }
    impl<'b> Func02<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 2> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01Case0(
                func02::Type01Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type01Case1(
                func02::Type01Case1::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 2> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02Case0(
                func02::Type02Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type02Case1(
                func02::Type02Case1::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type03_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type03(
                func02::Type03::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type04_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type04(
                func02::Type04::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 2>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x03 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type03_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x04 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type04_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Rocker Switch, 4 Rocker (F6-03)
    pub mod func03 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Light and Blind Control - Application Style 1 (F6-03-01), case 0
        #[derive(Clone, Copy)]
        pub struct Type01Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Rocker 1st action value.
            pub fn get_rocker_1st_action_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 3)
            }
            /// Get the Rocker 1st action value.
            pub fn get_rocker_1st_action(&self) -> Option<Type01Case0PropRocker1stAction> {
                let raw_value = self.get_rocker_1st_action_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy Bow value.
            pub fn get_energy_bow_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(3, 1)
            }
            /// Get the Energy Bow value.
            pub fn get_energy_bow(&self) -> Option<Type01Case0PropEnergyBow> {
                let raw_value = self.get_energy_bow_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Rocker 2nd action value.
            pub fn get_rocker_2nd_action_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 3)
            }
            /// Get the Rocker 2nd action value.
            pub fn get_rocker_2nd_action(&self) -> Option<Type01Case0PropRocker2ndAction> {
                let raw_value = self.get_rocker_2nd_action_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw 2nd Action value.
            pub fn get_2nd_action_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(7, 1)
            }
            /// Get the 2nd Action value.
            pub fn get_2nd_action(&self) -> Option<Type01Case0Prop2ndAction> {
                let raw_value = self.get_2nd_action_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01Case0")
                    .field("rocker_1st_action", &self.get_rocker_1st_action())
                    .field("energy_bow", &self.get_energy_bow())
                    .field("rocker_2nd_action", &self.get_rocker_2nd_action())
                    .field("2nd_action", &self.get_2nd_action())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case0PropRocker1stAction {
            ButtonAi = 0,
            ButtonA0 = 1,
            ButtonBi = 2,
            ButtonB0 = 3,
            ButtonCi = 4,
            ButtonC0 = 5,
            ButtonDi = 6,
            ButtonD0 = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case0PropEnergyBow {
            Released = false,
            Pressed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case0PropRocker2ndAction {
            ButtonAi = 0,
            ButtonA0 = 1,
            ButtonBi = 2,
            ButtonB0 = 3,
            ButtonCi = 4,
            ButtonC0 = 5,
            ButtonDi = 6,
            ButtonD0 = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case0Prop2ndAction {
            No2ndAction = false,
            _2ndActionValid = true,
            _Other(bool),
        }
        /// Light and Blind Control - Application Style 1 (F6-03-01), case 1
        #[derive(Clone, Copy)]
        pub struct Type01Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Number of buttons pressed simultaneously value.
            pub fn get_number_of_buttons_pressed_simultaneously_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 3)
            }
            /// Get the Number of buttons pressed simultaneously value.
            pub fn get_number_of_buttons_pressed_simultaneously(&self) -> Option<Type01Case1PropNumberOfButtonsPressedSimultaneously> {
                let raw_value = self.get_number_of_buttons_pressed_simultaneously_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy Bow value.
            pub fn get_energy_bow_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(3, 1)
            }
            /// Get the Energy Bow value.
            pub fn get_energy_bow(&self) -> Option<Type01Case1PropEnergyBow> {
                let raw_value = self.get_energy_bow_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01Case1")
                    .field("number_of_buttons_pressed_simultaneously", &self.get_number_of_buttons_pressed_simultaneously())
                    .field("energy_bow", &self.get_energy_bow())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case1PropNumberOfButtonsPressedSimultaneously {
            NoButtonPressed = 0,
            _2ButtonsPressed = 1,
            _3ButtonsPressed = 2,
            _4ButtonsPressed = 3,
            _5ButtonsPressed = 4,
            _6ButtonsPressed = 5,
            _7ButtonsPressed = 6,
            _8ButtonsPressed = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropEnergyBow {
            Released = false,
            Pressed = true,
            _Other(bool),
        }
        /// Light and Blind Control - Application Style 2 (F6-03-02), case 0
        #[derive(Clone, Copy)]
        pub struct Type02Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Rocker 1st action value.
            pub fn get_rocker_1st_action_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 3)
            }
            /// Get the Rocker 1st action value.
            pub fn get_rocker_1st_action(&self) -> Option<Type02Case0PropRocker1stAction> {
                let raw_value = self.get_rocker_1st_action_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy Bow value.
            pub fn get_energy_bow_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(3, 1)
            }
            /// Get the Energy Bow value.
            pub fn get_energy_bow(&self) -> Option<Type02Case0PropEnergyBow> {
                let raw_value = self.get_energy_bow_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Rocker 2nd action value.
            pub fn get_rocker_2nd_action_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 3)
            }
            /// Get the Rocker 2nd action value.
            pub fn get_rocker_2nd_action(&self) -> Option<Type02Case0PropRocker2ndAction> {
                let raw_value = self.get_rocker_2nd_action_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw 2nd Action value.
            pub fn get_2nd_action_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(7, 1)
            }
            /// Get the 2nd Action value.
            pub fn get_2nd_action(&self) -> Option<Type02Case0Prop2ndAction> {
                let raw_value = self.get_2nd_action_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02Case0")
                    .field("rocker_1st_action", &self.get_rocker_1st_action())
                    .field("energy_bow", &self.get_energy_bow())
                    .field("rocker_2nd_action", &self.get_rocker_2nd_action())
                    .field("2nd_action", &self.get_2nd_action())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type02Case0PropRocker1stAction {
            ButtonAi = 0,
            ButtonA0 = 1,
            ButtonBi = 2,
            ButtonB0 = 3,
            ButtonCi = 4,
            ButtonC0 = 5,
            ButtonDi = 6,
            ButtonD0 = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02Case0PropEnergyBow {
            Released = false,
            Pressed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type02Case0PropRocker2ndAction {
            ButtonAi = 0,
            ButtonA0 = 1,
            ButtonBi = 2,
            ButtonB0 = 3,
            ButtonCi = 4,
            ButtonC0 = 5,
            ButtonDi = 6,
            ButtonD0 = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02Case0Prop2ndAction {
            No2ndAction = false,
            _2ndActionValid = true,
            _Other(bool),
        }
        /// Light and Blind Control - Application Style 2 (F6-03-02), case 1
        #[derive(Clone, Copy)]
        pub struct Type02Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Number of buttons pressed simultaneously value.
            pub fn get_number_of_buttons_pressed_simultaneously_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 3)
            }
            /// Get the Number of buttons pressed simultaneously value.
            pub fn get_number_of_buttons_pressed_simultaneously(&self) -> Option<Type02Case1PropNumberOfButtonsPressedSimultaneously> {
                let raw_value = self.get_number_of_buttons_pressed_simultaneously_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy Bow value.
            pub fn get_energy_bow_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(3, 1)
            }
            /// Get the Energy Bow value.
            pub fn get_energy_bow(&self) -> Option<Type02Case1PropEnergyBow> {
                let raw_value = self.get_energy_bow_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02Case1")
                    .field("number_of_buttons_pressed_simultaneously", &self.get_number_of_buttons_pressed_simultaneously())
                    .field("energy_bow", &self.get_energy_bow())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type02Case1PropNumberOfButtonsPressedSimultaneously {
            NoButtonPressed = 0,
            _2ButtonsPressed = 1,
            _3ButtonsPressed = 2,
            _4ButtonsPressed = 3,
            _5ButtonsPressed = 4,
            _6ButtonsPressed = 5,
            _7ButtonsPressed = 6,
            _8ButtonsPressed = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02Case1PropEnergyBow {
            Released = false,
            Pressed = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func03<'b> {
        Type01Case0(func03::Type01Case0<'b>),
        Type01Case1(func03::Type01Case1<'b>),
        Type02Case0(func03::Type02Case0<'b>),
        Type02Case1(func03::Type02Case1<'b>),
    }
    impl<'b> Func03<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 2> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01Case0(
                func03::Type01Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type01Case1(
                func03::Type01Case1::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 2> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02Case0(
                func03::Type02Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type02Case1(
                func03::Type02Case1::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 2>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Position Switch, Home and Office Application (F6-04)
    pub mod func04 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Key Card Activated Switch (F6-04-01), case 0
        #[derive(Clone, Copy)]
        pub struct Type01Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Key Card value.
            pub fn get_key_card_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Key Card value.
            pub fn get_key_card(&self) -> Option<Type01Case0PropKeyCard> {
                let raw_value = self.get_key_card_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01Case0")
                    .field("key_card", &self.get_key_card())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case0PropKeyCard {
            Inserted0x70 = 112,
            _Other(u8),
        }
        /// Key Card Activated Switch (F6-04-01), case 1
        #[derive(Clone, Copy)]
        pub struct Type01Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Key Card value.
            pub fn get_key_card_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Key Card value.
            pub fn get_key_card(&self) -> Option<Type01Case1PropKeyCard> {
                let raw_value = self.get_key_card_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01Case1")
                    .field("key_card", &self.get_key_card())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case1PropKeyCard {
            TakenOut = 0,
            _Other(u8),
        }
        /// Key Card Activated Switch ERP2 (F6-04-02)
        #[derive(Clone, Copy)]
        pub struct Type02<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Energy Bow value.
            pub fn get_energy_bow_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(0, 1)
            }
            /// Get the Energy Bow value.
            pub fn get_energy_bow(&self) -> Option<Type02PropEnergyBow> {
                let raw_value = self.get_energy_bow_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Button coding value.
            pub fn get_button_coding_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(1, 1)
            }
            /// Get the Button coding value.
            pub fn get_button_coding(&self) -> Option<Type02PropButtonCoding> {
                let raw_value = self.get_button_coding_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw State of card value.
            pub fn get_state_of_card_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(5, 1)
            }
            /// Get the State of card value.
            pub fn get_state_of_card(&self) -> Option<Type02PropStateOfCard> {
                let raw_value = self.get_state_of_card_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02")
                    .field("energy_bow", &self.get_energy_bow())
                    .field("button_coding", &self.get_button_coding())
                    .field("state_of_card", &self.get_state_of_card())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropEnergyBow {
            TakenOut = false,
            CardInserted = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropButtonCoding {
            Button = false,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropStateOfCard {
            TakenOut = false,
            CardInserted = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func04<'b> {
        Type01Case0(func04::Type01Case0<'b>),
        Type01Case1(func04::Type01Case1<'b>),
        Type02(func04::Type02<'b>),
    }
    impl<'b> Func04<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 2> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01Case0(
                func04::Type01Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type01Case1(
                func04::Type01Case1::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02(
                func04::Type02::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 2>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Detectors (F6-05)
    pub mod func05 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Wind Speed Threshold Detector (F6-05-00)
        #[derive(Clone, Copy)]
        pub struct Type00<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Status value.
            pub fn get_status_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Status value.
            pub fn get_status(&self) -> Option<Type00PropStatus> {
                let raw_value = self.get_status_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00")
                    .field("status", &self.get_status())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropStatus {
            WindSpeedBelowThresholdAlarmOff = 0,
            WindSpeedExceedsThresholdAlarmOn = 16,
            EnergyLow = 48,
            _Other(u8),
        }
        /// Liquid Leakage Sensor (mechanic harvester) (F6-05-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Water sensor value.
            pub fn get_water_sensor_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Water sensor value.
            pub fn get_water_sensor(&self) -> Option<Type01PropWaterSensor> {
                let raw_value = self.get_water_sensor_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("water_sensor", &self.get_water_sensor())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropWaterSensor {
            WaterDetected = 17,
            _Other(u8),
        }
        /// Smoke Detector (F6-05-02)
        #[derive(Clone, Copy)]
        pub struct Type02<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Status value.
            pub fn get_status_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Status value.
            pub fn get_status(&self) -> Option<Type02PropStatus> {
                let raw_value = self.get_status_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02")
                    .field("status", &self.get_status())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type02PropStatus {
            SmokeAlarmOff = 0,
            SmokeAlarmOn = 16,
            EnergyLow = 48,
            _Other(u8),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func05<'b> {
        Type00(func05::Type00<'b>),
        Type01(func05::Type01<'b>),
        Type02(func05::Type02<'b>),
    }
    impl<'b> Func05<'b> {
        pub fn type00_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type00(
                func05::Type00::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func05::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02(
                func05::Type02::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x00 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type00_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Mechanical Handle (F6-10)
    pub mod func10 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Window Handle (F6-10-00)
        #[derive(Clone, Copy)]
        pub struct Type00<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Window handle value.
            pub fn get_window_handle_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Window handle value.
            pub fn get_window_handle(&self) -> Option<Type00PropWindowHandle> {
                let raw_value = self.get_window_handle_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00")
                    .field("window_handle", &self.get_window_handle())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropWindowHandle {
            _Other(u8),
        }
        /// Window Handle ERP2 (F6-10-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Handle coding value.
            pub fn get_handle_coding_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(1, 1)
            }
            /// Get the Handle coding value.
            pub fn get_handle_coding(&self) -> Option<Type01PropHandleCoding> {
                let raw_value = self.get_handle_coding_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Handle value value.
            pub fn get_handle_value_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Handle value value.
            pub fn get_handle_value(&self) -> Option<Type01PropHandleValue> {
                let raw_value = self.get_handle_value_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("handle_coding", &self.get_handle_coding())
                    .field("handle_value", &self.get_handle_value())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropHandleCoding {
            Handle = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropHandleValue {
            MovedFromRightToDown = 15,
            MovedFromLeftToUp = 13,
            MovedFromLeftToDown = 15,
            MovedFromRightToUp = 13,
            _Other(u8),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func10<'b> {
        Type00(func10::Type00<'b>),
        Type01(func10::Type01<'b>),
    }
    impl<'b> Func10<'b> {
        pub fn type00_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type00(
                func10::Type00::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func10::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x00 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type00_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RorgF6<'b> {
    Func01(rorgF6::Func01<'b>),
    Func02(rorgF6::Func02<'b>),
    Func03(rorgF6::Func03<'b>),
    Func04(rorgF6::Func04<'b>),
    Func05(rorgF6::Func05<'b>),
    Func10(rorgF6::Func10<'b>),
}
impl<'b> RorgF6<'b> {
    pub fn from_reversed_bytes(func_code: u8, type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 2>> {
        match func_code {
            0x01 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgF6::Func01::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func01(*f))
                        .peekable()
                ))
            },
            0x02 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgF6::Func02::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func02(*f))
                        .peekable()
                ))
            },
            0x03 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgF6::Func03::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func03(*f))
                        .peekable()
                ))
            },
            0x04 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgF6::Func04::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func04(*f))
                        .peekable()
                ))
            },
            0x05 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgF6::Func05::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func05(*f))
                        .peekable()
                ))
            },
            0x10 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgF6::Func10::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func10(*f))
                        .peekable()
                ))
            },
            _ => None,
        }
    }
}
/// 1BS Telegram (D5)
#[allow(non_snake_case)]
pub mod rorgD5 {
    #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
    /// Contacts and Switches (D5-00)
    pub mod func00 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Single Input Contact (D5-00-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Contact value.
            pub fn get_contact_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(7, 1)
            }
            /// Get the Contact value.
            pub fn get_contact(&self) -> Option<Type01PropContact> {
                let raw_value = self.get_contact_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Learn Button value.
            pub fn get_learn_button_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(4, 1)
            }
            /// Get the Learn Button value.
            pub fn get_learn_button(&self) -> Option<Type01PropLearnButton> {
                let raw_value = self.get_learn_button_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("contact", &self.get_contact())
                    .field("learn_button", &self.get_learn_button())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropContact {
            Open = false,
            Closed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropLearnButton {
            Pressed = false,
            NotPressed = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func00<'b> {
        Type01(func00::Type01<'b>),
    }
    impl<'b> Func00<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func00::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RorgD5<'b> {
    Func00(rorgD5::Func00<'b>),
}
impl<'b> RorgD5<'b> {
    pub fn from_reversed_bytes(func_code: u8, type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
        match func_code {
            0x00 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgD5::Func00::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func00(*f))
                        .peekable()
                ))
            },
            _ => None,
        }
    }
}
/// 4BS Telegram (A5)
#[allow(non_snake_case)]
pub mod rorgA5 {
    #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
    /// Temperature Sensors (A5-02)
    pub mod func02 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Temperature Sensor Range -40°C to 0°C (A5-02-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type01PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, -40.0, 0.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range -30°C to +10°C (A5-02-02)
        #[derive(Clone, Copy)]
        pub struct Type02<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type02PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, -30.0, 10.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type02<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range -20°C to +20°C (A5-02-03)
        #[derive(Clone, Copy)]
        pub struct Type03<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type03<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type03PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, -20.0, 20.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type03<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type03")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range -10°C to +30°C (A5-02-04)
        #[derive(Clone, Copy)]
        pub struct Type04<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type04<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type04PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, -10.0, 30.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type04<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type04")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range 0°C to +40°C (A5-02-05)
        #[derive(Clone, Copy)]
        pub struct Type05<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type05<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type05PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 0.0, 40.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type05<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type05")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range +10°C to +50°C (A5-02-06)
        #[derive(Clone, Copy)]
        pub struct Type06<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type06<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type06PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 10.0, 50.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type06<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type06")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range +20°C to +60°C (A5-02-07)
        #[derive(Clone, Copy)]
        pub struct Type07<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type07<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type07PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 20.0, 60.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type07<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type07")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type07PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range +30°C to +70°C (A5-02-08)
        #[derive(Clone, Copy)]
        pub struct Type08<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type08<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type08PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 30.0, 70.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type08<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type08")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range +40°C to +80°C (A5-02-09)
        #[derive(Clone, Copy)]
        pub struct Type09<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type09<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type09PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 40.0, 80.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type09<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type09")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type09PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range +50°C to +90°C (A5-02-0A)
        #[derive(Clone, Copy)]
        pub struct Type0A<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type0A<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type0APropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 50.0, 90.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type0A<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type0A")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0APropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range +60°C to +100°C (A5-02-0B)
        #[derive(Clone, Copy)]
        pub struct Type0B<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type0B<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type0BPropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 60.0, 100.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type0B<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type0B")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0BPropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range -60°C to +20°C (A5-02-10)
        #[derive(Clone, Copy)]
        pub struct Type10<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type10<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type10PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, -60.0, 20.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type10<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type10")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type10PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range -50°C to +30°C (A5-02-11)
        #[derive(Clone, Copy)]
        pub struct Type11<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type11<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type11PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, -50.0, 30.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type11<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type11")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type11PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range -40°C to +40°C (A5-02-12)
        #[derive(Clone, Copy)]
        pub struct Type12<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type12<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type12PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, -40.0, 40.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type12<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type12")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type12PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range -30°C to +50°C (A5-02-13)
        #[derive(Clone, Copy)]
        pub struct Type13<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type13<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type13PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, -30.0, 50.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type13<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type13")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type13PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range -20°C to +60°C (A5-02-14)
        #[derive(Clone, Copy)]
        pub struct Type14<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type14<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type14PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, -20.0, 60.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type14<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type14")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type14PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range -10°C to +70°C (A5-02-15)
        #[derive(Clone, Copy)]
        pub struct Type15<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type15<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type15PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, -10.0, 70.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type15<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type15")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type15PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range 0°C to +80°C (A5-02-16)
        #[derive(Clone, Copy)]
        pub struct Type16<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type16<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type16PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 0.0, 80.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type16<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type16")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type16PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range +10°C to +90°C (A5-02-17)
        #[derive(Clone, Copy)]
        pub struct Type17<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type17<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type17PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 10.0, 90.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type17<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type17")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type17PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range +20°C to +100°C (A5-02-18)
        #[derive(Clone, Copy)]
        pub struct Type18<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type18<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type18PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 20.0, 100.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type18<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type18")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type18PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range +30°C to +110°C (A5-02-19)
        #[derive(Clone, Copy)]
        pub struct Type19<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type19<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type19PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 30.0, 110.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type19<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type19")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type19PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range +40°C to +120°C (A5-02-1A)
        #[derive(Clone, Copy)]
        pub struct Type1A<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type1A<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type1APropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 40.0, 120.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type1A<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type1A")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type1APropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Temperature Sensor Range +50°C to +130°C (A5-02-1B)
        #[derive(Clone, Copy)]
        pub struct Type1B<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type1B<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type1BPropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 50.0, 130.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type1B<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type1B")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type1BPropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// 10 Bit Temperature Sensor Range -10°C to +41.2°C (A5-02-20)
        #[derive(Clone, Copy)]
        pub struct Type20<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type20<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type20PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(14, 10)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 1023.0, 0.0, -10.0, 41.2))
            }
        }
        impl<'b> ::core::fmt::Debug for Type20<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type20")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type20PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// 10 Bit Temperature Sensor Range -40°C to +62.3°C (A5-02-30)
        #[derive(Clone, Copy)]
        pub struct Type30<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type30<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type30PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(14, 10)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 1023.0, 0.0, -40.0, 62.3))
            }
        }
        impl<'b> ::core::fmt::Debug for Type30<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type30")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type30PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func02<'b> {
        Type01(func02::Type01<'b>),
        Type02(func02::Type02<'b>),
        Type03(func02::Type03<'b>),
        Type04(func02::Type04<'b>),
        Type05(func02::Type05<'b>),
        Type06(func02::Type06<'b>),
        Type07(func02::Type07<'b>),
        Type08(func02::Type08<'b>),
        Type09(func02::Type09<'b>),
        Type0A(func02::Type0A<'b>),
        Type0B(func02::Type0B<'b>),
        Type10(func02::Type10<'b>),
        Type11(func02::Type11<'b>),
        Type12(func02::Type12<'b>),
        Type13(func02::Type13<'b>),
        Type14(func02::Type14<'b>),
        Type15(func02::Type15<'b>),
        Type16(func02::Type16<'b>),
        Type17(func02::Type17<'b>),
        Type18(func02::Type18<'b>),
        Type19(func02::Type19<'b>),
        Type1A(func02::Type1A<'b>),
        Type1B(func02::Type1B<'b>),
        Type20(func02::Type20<'b>),
        Type30(func02::Type30<'b>),
    }
    impl<'b> Func02<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func02::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02(
                func02::Type02::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type03_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type03(
                func02::Type03::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type04_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type04(
                func02::Type04::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type05_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type05(
                func02::Type05::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type06_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type06(
                func02::Type06::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type07_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type07(
                func02::Type07::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type08_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type08(
                func02::Type08::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type09_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type09(
                func02::Type09::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type0A_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type0A(
                func02::Type0A::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type0B_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type0B(
                func02::Type0B::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type10_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type10(
                func02::Type10::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type11_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type11(
                func02::Type11::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type12_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type12(
                func02::Type12::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type13_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type13(
                func02::Type13::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type14_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type14(
                func02::Type14::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type15_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type15(
                func02::Type15::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type16_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type16(
                func02::Type16::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type17_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type17(
                func02::Type17::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type18_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type18(
                func02::Type18::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type19_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type19(
                func02::Type19::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type1A_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type1A(
                func02::Type1A::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type1B_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type1B(
                func02::Type1B::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type20_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type20(
                func02::Type20::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type30_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type30(
                func02::Type30::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x03 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type03_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x04 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type04_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x05 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type05_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x06 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type06_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x07 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type07_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x08 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type08_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x09 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type09_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x0A => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type0A_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x0B => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type0B_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x10 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type10_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x11 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type11_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x12 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type12_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x13 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type13_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x14 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type14_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x15 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type15_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x16 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type16_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x17 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type17_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x18 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type18_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x19 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type19_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x1A => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type1A_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x1B => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type1B_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x20 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type20_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x30 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type30_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Light Sensor (A5-06)
    pub mod func06 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Range 300lx to 60.000lx (A5-06-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type01PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 5.1))
            }

            /// Get the raw Illumination value.
            pub fn get_illumination_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Illumination value in units of lx.
            pub fn get_illumination(&self) -> Option<f64> {
                let raw_value = self.get_illumination_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 300.0, 30000.0))
            }

            /// Get the raw Illumination 1 value.
            pub fn get_illumination_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Illumination 1 value in units of lx.
            pub fn get_illumination_1(&self) -> Option<f64> {
                let raw_value = self.get_illumination_1_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 600.0, 60000.0))
            }

            /// Get the raw Range select value.
            pub fn get_range_select_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Range select value.
            pub fn get_range_select(&self) -> Option<Type01PropRangeSelect> {
                let raw_value = self.get_range_select_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("illumination", &self.get_illumination())
                    .field("illumination_1", &self.get_illumination_1())
                    .field("range_select", &self.get_range_select())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropRangeSelect {
            RangeAccToDb_1Ill1 = false,
            RangeAccToDb_2Ill2 = true,
            _Other(bool),
        }
        /// Range 0lx to 1.020lx (A5-06-02)
        #[derive(Clone, Copy)]
        pub struct Type02<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type02PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 5.1))
            }

            /// Get the raw Illumination value.
            pub fn get_illumination_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Illumination value in units of lx.
            pub fn get_illumination(&self) -> Option<f64> {
                let raw_value = self.get_illumination_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 510.0))
            }

            /// Get the raw Illumination 1 value.
            pub fn get_illumination_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Illumination 1 value in units of lx.
            pub fn get_illumination_1(&self) -> Option<f64> {
                let raw_value = self.get_illumination_1_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 1020.0))
            }

            /// Get the raw Range select value.
            pub fn get_range_select_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Range select value.
            pub fn get_range_select(&self) -> Option<Type02PropRangeSelect> {
                let raw_value = self.get_range_select_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("illumination", &self.get_illumination())
                    .field("illumination_1", &self.get_illumination_1())
                    .field("range_select", &self.get_range_select())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropRangeSelect {
            RangeAccToDb_1Ill1 = false,
            RangeAccToDb_2Ill2 = true,
            _Other(bool),
        }
        /// 10-bit measurement (1-Lux resolution) with range 0lx to 1000lx (A5-06-03)
        #[derive(Clone, Copy)]
        pub struct Type03<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type03<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type03PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 5.0))
            }

            /// Get the raw Illumination value.
            pub fn get_illumination_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(8, 10)
            }
            /// Get the Illumination value in units of lx.
            pub fn get_illumination(&self) -> Option<f64> {
                let raw_value = self.get_illumination_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 1000.0, 0.0, 1000.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type03<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type03")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("illumination", &self.get_illumination())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Curtain Wall Brightness Sensor (A5-06-04)
        #[derive(Clone, Copy)]
        pub struct Type04<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type04<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type04PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, -20.0, 60.0))
            }

            /// Get the raw Illuminance value.
            pub fn get_illuminance_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(8, 16)
            }
            /// Get the Illuminance value in units of lx.
            pub fn get_illuminance(&self) -> Option<f64> {
                let raw_value = self.get_illuminance_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 65535.0, 0.0, 65535.0))
            }

            /// Get the raw Energy Storage value.
            pub fn get_energy_storage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Energy Storage value in units of %.
            pub fn get_energy_storage(&self) -> Option<f64> {
                let raw_value = self.get_energy_storage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 15.0, 0.0, 100.0))
            }

            /// Get the raw Temperature Availability value.
            pub fn get_temperature_availability_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Temperature Availability value.
            pub fn get_temperature_availability(&self) -> Option<Type04PropTemperatureAvailability> {
                let raw_value = self.get_temperature_availability_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy Storage Availability value.
            pub fn get_energy_storage_availability_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Energy Storage Availability value.
            pub fn get_energy_storage_availability(&self) -> Option<Type04PropEnergyStorageAvailability> {
                let raw_value = self.get_energy_storage_availability_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type04<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type04")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .field("illuminance", &self.get_illuminance())
                    .field("energy_storage", &self.get_energy_storage())
                    .field("temperature_availability", &self.get_temperature_availability())
                    .field("energy_storage_availability", &self.get_energy_storage_availability())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropTemperatureAvailability {
            TemperatureDataIsUnavailable = false,
            TemperatureDataIsAvailable = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropEnergyStorageAvailability {
            EnergyStorageDataIsUnavailable = false,
            EnergyStorageDataIsAvailable = true,
            _Other(bool),
        }
        /// Range 0lx to 10.200lx (A5-06-05)
        #[derive(Clone, Copy)]
        pub struct Type05<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type05<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type05PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 5.1))
            }

            /// Get the raw Illumination value.
            pub fn get_illumination_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Illumination value in units of lx.
            pub fn get_illumination(&self) -> Option<f64> {
                let raw_value = self.get_illumination_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 5100.0))
            }

            /// Get the raw Illumination 1 value.
            pub fn get_illumination_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Illumination 1 value in units of lx.
            pub fn get_illumination_1(&self) -> Option<f64> {
                let raw_value = self.get_illumination_1_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 10200.0))
            }

            /// Get the raw Range select value.
            pub fn get_range_select_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Range select value.
            pub fn get_range_select(&self) -> Option<Type05PropRangeSelect> {
                let raw_value = self.get_range_select_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type05<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type05")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("illumination", &self.get_illumination())
                    .field("illumination_1", &self.get_illumination_1())
                    .field("range_select", &self.get_range_select())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropRangeSelect {
            RangeAccToDb_1Ill1 = false,
            RangeAccToDb_2Ill2 = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func06<'b> {
        Type01(func06::Type01<'b>),
        Type02(func06::Type02<'b>),
        Type03(func06::Type03<'b>),
        Type04(func06::Type04<'b>),
        Type05(func06::Type05<'b>),
    }
    impl<'b> Func06<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func06::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02(
                func06::Type02::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type03_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type03(
                func06::Type03::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type04_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type04(
                func06::Type04::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type05_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type05(
                func06::Type05::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x03 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type03_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x04 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type04_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x05 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type05_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Occupancy Sensor (A5-07)
    pub mod func07 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Occupancy with Supply voltage monitor (A5-07-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type01PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage (OPTIONAL) value.
            pub fn get_supply_voltage_optional_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage (OPTIONAL) value in units of V.
            pub fn get_supply_voltage_optional(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_optional_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 5.0))
            }

            /// Get the raw PIR Status value.
            pub fn get_pir_status_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the PIR Status value.
            pub fn get_pir_status(&self) -> Option<Type01PropPirStatus> {
                let raw_value = self.get_pir_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage availability value.
            pub fn get_supply_voltage_availability_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Supply voltage availability value.
            pub fn get_supply_voltage_availability(&self) -> Option<Type01PropSupplyVoltageAvailability> {
                let raw_value = self.get_supply_voltage_availability_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage_optional", &self.get_supply_voltage_optional())
                    .field("pir_status", &self.get_pir_status())
                    .field("supply_voltage_availability", &self.get_supply_voltage_availability())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropPirStatus {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropSupplyVoltageAvailability {
            SupplyVoltageIsNotSupported = false,
            SupplyVoltageIsSupported = true,
            _Other(bool),
        }
        /// Occupancy with Supply voltage monitor (A5-07-02)
        #[derive(Clone, Copy)]
        pub struct Type02<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type02PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage (REQUIRED) value.
            pub fn get_supply_voltage_required_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage (REQUIRED) value in units of V.
            pub fn get_supply_voltage_required(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_required_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 5.0))
            }

            /// Get the raw PIR Status value.
            pub fn get_pir_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(24, 1)
            }
            /// Get the PIR Status value.
            pub fn get_pir_status(&self) -> Option<Type02PropPirStatus> {
                let raw_value = self.get_pir_status_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage_required", &self.get_supply_voltage_required())
                    .field("pir_status", &self.get_pir_status())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropPirStatus {
            UncertainOfOccupancyStatus = false,
            MotionDetected = true,
            _Other(bool),
        }
        /// Occupancy with Supply voltage monitor and 10-bit illumination measurement (A5-07-03)
        #[derive(Clone, Copy)]
        pub struct Type03<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type03<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type03PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage (REQUIRED) value.
            pub fn get_supply_voltage_required_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage (REQUIRED) value in units of V.
            pub fn get_supply_voltage_required(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_required_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 5.0))
            }

            /// Get the raw Illumination value.
            pub fn get_illumination_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(8, 10)
            }
            /// Get the Illumination value in units of lx.
            pub fn get_illumination(&self) -> Option<f64> {
                let raw_value = self.get_illumination_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 1000.0, 0.0, 1000.0))
            }

            /// Get the raw PIR Status value.
            pub fn get_pir_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(24, 1)
            }
            /// Get the PIR Status value.
            pub fn get_pir_status(&self) -> Option<Type03PropPirStatus> {
                let raw_value = self.get_pir_status_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type03<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type03")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage_required", &self.get_supply_voltage_required())
                    .field("illumination", &self.get_illumination())
                    .field("pir_status", &self.get_pir_status())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropPirStatus {
            MotionDetected = true,
            UncertainOfOccupancyStatus = false,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func07<'b> {
        Type01(func07::Type01<'b>),
        Type02(func07::Type02<'b>),
        Type03(func07::Type03<'b>),
    }
    impl<'b> Func07<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func07::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02(
                func07::Type02::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type03_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type03(
                func07::Type03::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x03 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type03_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Light, Temperature and Occupancy Sensor (A5-08)
    pub mod func08 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Range 0lx to 510lx, 0°C to +51°C and Occupancy Button (A5-08-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type01PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 5.1))
            }

            /// Get the raw Illumination value.
            pub fn get_illumination_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Illumination value in units of lx.
            pub fn get_illumination(&self) -> Option<f64> {
                let raw_value = self.get_illumination_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 510.0))
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 51.0))
            }

            /// Get the raw PIR Status value.
            pub fn get_pir_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the PIR Status value.
            pub fn get_pir_status(&self) -> Option<Type01PropPirStatus> {
                let raw_value = self.get_pir_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Occupancy Button value.
            pub fn get_occupancy_button_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Occupancy Button value.
            pub fn get_occupancy_button(&self) -> Option<Type01PropOccupancyButton> {
                let raw_value = self.get_occupancy_button_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("illumination", &self.get_illumination())
                    .field("temperature", &self.get_temperature())
                    .field("pir_status", &self.get_pir_status())
                    .field("occupancy_button", &self.get_occupancy_button())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropPirStatus {
            PirOn = false,
            PirOff = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropOccupancyButton {
            ButtonPressed = false,
            ButtonReleased = true,
            _Other(bool),
        }
        /// Range 0lx to 1020lx, 0°C to +51°C and Occupancy Button (A5-08-02)
        #[derive(Clone, Copy)]
        pub struct Type02<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type02PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 5.1))
            }

            /// Get the raw Illumination value.
            pub fn get_illumination_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Illumination value in units of lx.
            pub fn get_illumination(&self) -> Option<f64> {
                let raw_value = self.get_illumination_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 1020.0))
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 51.0))
            }

            /// Get the raw PIR Status value.
            pub fn get_pir_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the PIR Status value.
            pub fn get_pir_status(&self) -> Option<Type02PropPirStatus> {
                let raw_value = self.get_pir_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Occupancy Button value.
            pub fn get_occupancy_button_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Occupancy Button value.
            pub fn get_occupancy_button(&self) -> Option<Type02PropOccupancyButton> {
                let raw_value = self.get_occupancy_button_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("illumination", &self.get_illumination())
                    .field("temperature", &self.get_temperature())
                    .field("pir_status", &self.get_pir_status())
                    .field("occupancy_button", &self.get_occupancy_button())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropPirStatus {
            PirOn = false,
            PirOff = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropOccupancyButton {
            ButtonPressed = false,
            ButtonReleased = true,
            _Other(bool),
        }
        /// Range 0lx to 1530lx, -30°C to +50°C and Occupancy Button (A5-08-03)
        #[derive(Clone, Copy)]
        pub struct Type03<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type03<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type03PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 5.1))
            }

            /// Get the raw Illumination value.
            pub fn get_illumination_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Illumination value in units of lx.
            pub fn get_illumination(&self) -> Option<f64> {
                let raw_value = self.get_illumination_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 1530.0))
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, -30.0, 50.0))
            }

            /// Get the raw PIR Status value.
            pub fn get_pir_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the PIR Status value.
            pub fn get_pir_status(&self) -> Option<Type03PropPirStatus> {
                let raw_value = self.get_pir_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Occupancy Button value.
            pub fn get_occupancy_button_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Occupancy Button value.
            pub fn get_occupancy_button(&self) -> Option<Type03PropOccupancyButton> {
                let raw_value = self.get_occupancy_button_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type03<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type03")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("illumination", &self.get_illumination())
                    .field("temperature", &self.get_temperature())
                    .field("pir_status", &self.get_pir_status())
                    .field("occupancy_button", &self.get_occupancy_button())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropPirStatus {
            PirOn = false,
            PirOff = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropOccupancyButton {
            ButtonPressed = false,
            ButtonReleased = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func08<'b> {
        Type01(func08::Type01<'b>),
        Type02(func08::Type02<'b>),
        Type03(func08::Type03<'b>),
    }
    impl<'b> Func08<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func08::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02(
                func08::Type02::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type03_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type03(
                func08::Type03::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x03 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type03_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Gas Sensor (A5-09)
    pub mod func09 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// CO-Sensor 0 ppm to 1020 ppm (A5-09-02)
        #[derive(Clone, Copy)]
        pub struct Type02<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type02PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 5.1))
            }

            /// Get the raw Concentration value.
            pub fn get_concentration_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Concentration value in units of ppm.
            pub fn get_concentration(&self) -> Option<f64> {
                let raw_value = self.get_concentration_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 1020.0))
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 51.0))
            }

            /// Get the raw T-Sensor value.
            pub fn get_t_sensor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the T-Sensor value.
            pub fn get_t_sensor(&self) -> Option<Type02PropTSensor> {
                let raw_value = self.get_t_sensor_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("concentration", &self.get_concentration())
                    .field("temperature", &self.get_temperature())
                    .field("t_sensor", &self.get_t_sensor())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropTSensor {
            TemperatureSensorNotAvailable = false,
            TemperatureSensorAvailable = true,
            _Other(bool),
        }
        /// CO2 Sensor (A5-09-04)
        #[derive(Clone, Copy)]
        pub struct Type04<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type04<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type04PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Humidity value.
            pub fn get_humidity_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Humidity value in units of %.
            pub fn get_humidity(&self) -> Option<f64> {
                let raw_value = self.get_humidity_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 200.0, 0.0, 100.0))
            }

            /// Get the raw Concentration value.
            pub fn get_concentration_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Concentration value in units of ppm.
            pub fn get_concentration(&self) -> Option<f64> {
                let raw_value = self.get_concentration_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 2550.0))
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 51.0))
            }

            /// Get the raw H-Sensor value.
            pub fn get_h_sensor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the H-Sensor value.
            pub fn get_h_sensor(&self) -> Option<Type04PropHSensor> {
                let raw_value = self.get_h_sensor_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw T-Sensor value.
            pub fn get_t_sensor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the T-Sensor value.
            pub fn get_t_sensor(&self) -> Option<Type04PropTSensor> {
                let raw_value = self.get_t_sensor_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type04<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type04")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("humidity", &self.get_humidity())
                    .field("concentration", &self.get_concentration())
                    .field("temperature", &self.get_temperature())
                    .field("h_sensor", &self.get_h_sensor())
                    .field("t_sensor", &self.get_t_sensor())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropHSensor {
            HumiditySensorNotAvailable = false,
            HumiditySensorAvailable = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropTSensor {
            TemperatureSensorNotAvailable = false,
            TemperatureSensorAvailable = true,
            _Other(bool),
        }
        /// VOC Sensor (A5-09-05)
        #[derive(Clone, Copy)]
        pub struct Type05<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type05<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw VOC value.
            pub fn get_voc_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(0, 16)
            }
            /// Get the VOC value in units of ppb.
            pub fn get_voc(&self) -> Option<f64> {
                let raw_value = self.get_voc_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 65535.0, 0.0, 65535.0))
            }

            /// Get the raw VOC ID value.
            pub fn get_voc_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the VOC ID value.
            pub fn get_voc_id(&self) -> Option<Type05PropVocId> {
                let raw_value = self.get_voc_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type05PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Scale Multiplier value.
            pub fn get_scale_multiplier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(30, 2)
            }
            /// Get the Scale Multiplier value.
            pub fn get_scale_multiplier(&self) -> Option<Type05PropScaleMultiplier> {
                let raw_value = self.get_scale_multiplier_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type05<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type05")
                    .field("voc", &self.get_voc())
                    .field("voc_id", &self.get_voc_id())
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("scale_multiplier", &self.get_scale_multiplier())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type05PropVocId {
            VoctTotal = 0,
            Formaldehyde = 1,
            Benzene = 2,
            Styrene = 3,
            Toluene = 4,
            Tetrachloroethylene = 5,
            Xylene = 6,
            NHexane = 7,
            NOctane = 8,
            Cyclopentane = 9,
            Methanol = 10,
            Ethanol = 11,
            _1Pentanol = 12,
            Acetone = 13,
            EthyleneOxide = 14,
            AcetaldehydeUe = 15,
            AceticAcid = 16,
            PropioniceAcid = 17,
            ValericAcid = 18,
            ButyricAcid = 19,
            Ammoniac = 20,
            HydrogenSulfide = 22,
            Dimethylsulfide = 23,
            _2ButanolButylAlcohol = 24,
            _2Methylpropanol = 25,
            DiethylEther = 26,
            Ozone = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type05PropScaleMultiplier {
            _001 = 0,
            _01 = 1,
            _1 = 2,
            _10 = 3,
            _Other(u8),
        }
        /// Radon (A5-09-06)
        #[derive(Clone, Copy)]
        pub struct Type06<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type06<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Radon value.
            pub fn get_radon_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(0, 10)
            }
            /// Get the Radon value in units of Bq/m3.
            pub fn get_radon(&self) -> Option<f64> {
                let raw_value = self.get_radon_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 1023.0, 0.0, 1023.0))
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type06PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type06<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type06")
                    .field("radon", &self.get_radon())
                    .field("lrn_bit", &self.get_lrn_bit())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Particles (A5-09-07)
        #[derive(Clone, Copy)]
        pub struct Type07<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type07<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Particles_10 value.
            pub fn get_particles_10_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(0, 9)
            }
            /// Get the Particles_10 value in units of µg/m3.
            pub fn get_particles_10(&self) -> Option<f64> {
                let raw_value = self.get_particles_10_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 511.0, 0.0, 511.0))
            }

            /// Get the raw Particles_2.5 value.
            pub fn get_particles_2_5_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(9, 9)
            }
            /// Get the Particles_2.5 value in units of µg/m3.
            pub fn get_particles_2_5(&self) -> Option<f64> {
                let raw_value = self.get_particles_2_5_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 511.0, 0.0, 511.0))
            }

            /// Get the raw Particles_1 value.
            pub fn get_particles_1_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(18, 9)
            }
            /// Get the Particles_1 value in units of µg/m3.
            pub fn get_particles_1(&self) -> Option<f64> {
                let raw_value = self.get_particles_1_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 511.0, 0.0, 511.0))
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type07PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw PM10 active value.
            pub fn get_pm10_active_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the PM10 active value.
            pub fn get_pm10_active(&self) -> Option<Type07PropPm10Active> {
                let raw_value = self.get_pm10_active_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw PM2.5 active value.
            pub fn get_pm2_5_active_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the PM2.5 active value.
            pub fn get_pm2_5_active(&self) -> Option<Type07PropPm25Active> {
                let raw_value = self.get_pm2_5_active_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw PM1 active value.
            pub fn get_pm1_active_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the PM1 active value.
            pub fn get_pm1_active(&self) -> Option<Type07PropPm1Active> {
                let raw_value = self.get_pm1_active_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type07<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type07")
                    .field("particles_10", &self.get_particles_10())
                    .field("particles_2_5", &self.get_particles_2_5())
                    .field("particles_1", &self.get_particles_1())
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("pm10_active", &self.get_pm10_active())
                    .field("pm2_5_active", &self.get_pm2_5_active())
                    .field("pm1_active", &self.get_pm1_active())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type07PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type07PropPm10Active {
            Pm10NotActive = false,
            Pm10Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type07PropPm25Active {
            Pm25NotActive = false,
            Pm25Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type07PropPm1Active {
            Pm1NotActive = false,
            Pm1Active = true,
            _Other(bool),
        }
        /// Pure CO2 Sensor (A5-09-08)
        #[derive(Clone, Copy)]
        pub struct Type08<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type08<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw CO2 value.
            pub fn get_co2_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the CO2 value in units of ppm.
            pub fn get_co2(&self) -> Option<f64> {
                let raw_value = self.get_co2_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 2000.0))
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type08PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type08<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type08")
                    .field("co2", &self.get_co2())
                    .field("lrn_bit", &self.get_lrn_bit())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Pure CO2 Sensor with Power Failure Detection (A5-09-09)
        #[derive(Clone, Copy)]
        pub struct Type09<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type09<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw CO2 value.
            pub fn get_co2_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the CO2 value in units of ppm.
            pub fn get_co2(&self) -> Option<f64> {
                let raw_value = self.get_co2_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 2000.0))
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type09PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Power Failure detection value.
            pub fn get_power_failure_detection_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Power Failure detection value.
            pub fn get_power_failure_detection(&self) -> Option<Type09PropPowerFailureDetection> {
                let raw_value = self.get_power_failure_detection_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type09<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type09")
                    .field("co2", &self.get_co2())
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("power_failure_detection", &self.get_power_failure_detection())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type09PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type09PropPowerFailureDetection {
            PowerFailureNotDetected = false,
            PowerFailureDetected = true,
            _Other(bool),
        }
        /// Hydrogen Gas Sensor (A5-09-0A)
        #[derive(Clone, Copy)]
        pub struct Type0A<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type0A<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Concentration value.
            pub fn get_concentration_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(0, 16)
            }
            /// Get the Concentration value in units of ppm.
            pub fn get_concentration(&self) -> Option<f64> {
                let raw_value = self.get_concentration_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 65535.0, 0.0, 65535.0))
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, -20.0, 60.0))
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 15.0, 2.0, 5.0))
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type0APropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temp sensor availability value.
            pub fn get_temp_sensor_availability_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Temp sensor availability value.
            pub fn get_temp_sensor_availability(&self) -> Option<Type0APropTempSensorAvailability> {
                let raw_value = self.get_temp_sensor_availability_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage availability value.
            pub fn get_supply_voltage_availability_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Supply voltage availability value.
            pub fn get_supply_voltage_availability(&self) -> Option<Type0APropSupplyVoltageAvailability> {
                let raw_value = self.get_supply_voltage_availability_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type0A<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type0A")
                    .field("concentration", &self.get_concentration())
                    .field("temperature", &self.get_temperature())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temp_sensor_availability", &self.get_temp_sensor_availability())
                    .field("supply_voltage_availability", &self.get_supply_voltage_availability())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0APropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0APropTempSensorAvailability {
            TempSensorIsNotSupported = false,
            TempSensorIsSupported = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0APropSupplyVoltageAvailability {
            SupplyVoltageIsNotSupported = false,
            SupplyVoltageIsSupported = true,
            _Other(bool),
        }
        /// Radioactivity Sensor (A5-09-0B)
        #[derive(Clone, Copy)]
        pub struct Type0B<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type0B<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 15.0, 2.0, 5.0))
            }

            /// Get the raw Radioactivity value.
            pub fn get_radioactivity_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(8, 16)
            }
            /// Get the Radioactivity value in units of According to.
            pub fn get_radioactivity(&self) -> Option<f64> {
                let raw_value = self.get_radioactivity_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 65535.0, 0.0, 6553.0))
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type0BPropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Scale Multiplier value.
            pub fn get_scale_multiplier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Scale Multiplier value.
            pub fn get_scale_multiplier(&self) -> Option<Type0BPropScaleMultiplier> {
                let raw_value = self.get_scale_multiplier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Value unit value.
            pub fn get_value_unit_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(29, 2)
            }
            /// Get the Value unit value.
            pub fn get_value_unit(&self) -> Option<Type0BPropValueUnit> {
                let raw_value = self.get_value_unit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage availability value.
            pub fn get_supply_voltage_availability_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Supply voltage availability value.
            pub fn get_supply_voltage_availability(&self) -> Option<Type0BPropSupplyVoltageAvailability> {
                let raw_value = self.get_supply_voltage_availability_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type0B<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type0B")
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("radioactivity", &self.get_radioactivity())
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("scale_multiplier", &self.get_scale_multiplier())
                    .field("value_unit", &self.get_value_unit())
                    .field("supply_voltage_availability", &self.get_supply_voltage_availability())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0BPropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type0BPropScaleMultiplier {
            _0001 = 0,
            _001 = 1,
            _01 = 2,
            _1 = 3,
            _10 = 4,
            _100 = 5,
            _1000 = 6,
            _10000 = 7,
            _100000 = 8,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type0BPropValueUnit {
            MuSvH = 0,
            Cpm = 1,
            BqL = 2,
            BqKg = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0BPropSupplyVoltageAvailability {
            SupplyVoltageIsNotSupported = false,
            SupplyVoltageIsSupported = true,
            _Other(bool),
        }
        /// VOC Sensor (A5-09-0C)
        #[derive(Clone, Copy)]
        pub struct Type0C<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type0C<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type0CPropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw VOC value.
            pub fn get_voc_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(0, 16)
            }
            /// Get the VOC value.
            pub fn get_voc(&self) -> Option<f64> {
                let raw_value = self.get_voc_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 65535.0, 0.0, 65535.0))
            }

            /// Get the raw VOC ID* value.
            pub fn get_voc_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the VOC ID* value.
            pub fn get_voc_id(&self) -> Option<Type0CPropVocId> {
                let raw_value = self.get_voc_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit value.
            pub fn get_unit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Unit value.
            pub fn get_unit(&self) -> Option<Type0CPropUnit> {
                let raw_value = self.get_unit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Scale Multiplier value.
            pub fn get_scale_multiplier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(30, 2)
            }
            /// Get the Scale Multiplier value.
            pub fn get_scale_multiplier(&self) -> Option<Type0CPropScaleMultiplier> {
                let raw_value = self.get_scale_multiplier_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type0C<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type0C")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("voc", &self.get_voc())
                    .field("voc_id", &self.get_voc_id())
                    .field("unit", &self.get_unit())
                    .field("scale_multiplier", &self.get_scale_multiplier())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0CPropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type0CPropVocId {
            VoctTotal = 0,
            Formaldehyde = 1,
            Benzene = 2,
            Styrene = 3,
            Toluene = 4,
            Tetrachloroethylene = 5,
            Xylene = 6,
            NHexane = 7,
            NOctane = 8,
            Cyclopentane = 9,
            Methanol = 10,
            Ethanol = 11,
            _1Pentanol = 12,
            Acetone = 13,
            EthyleneOxide = 14,
            AcetaldehydeUe = 15,
            AceticAcid = 16,
            PropioniceAcid = 17,
            ValericAcid = 18,
            ButyricAcid = 19,
            Ammoniac = 20,
            HydrogenSulfide = 22,
            Dimethylsulfide = 23,
            _2ButanolButylAlcohol = 24,
            _2Methylpropanol = 25,
            DiethylEther = 26,
            Naphthalene = 27,
            _4Phenylcyclohexene = 28,
            Limonene = 29,
            Trichloroethylene = 30,
            IsovalericAcid = 31,
            Indole = 32,
            Cadaverine = 33,
            Putrescine = 34,
            CaproicAcid = 35,
            Ozone = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0CPropUnit {
            Ppb = false,
            MuGM3 = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type0CPropScaleMultiplier {
            _001 = 0,
            _01 = 1,
            _1 = 2,
            _10 = 3,
            _Other(u8),
        }
        /// VOC, humidity  (A5-09-0D)
        #[derive(Clone, Copy)]
        pub struct Type0D<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type0D<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw humidity value.
            pub fn get_humidity_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the humidity value.
            pub fn get_humidity(&self) -> Option<Type0DPropHumidity> {
                let raw_value = self.get_humidity_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw TVOC value.
            pub fn get_tvoc_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the TVOC value.
            pub fn get_tvoc(&self) -> Option<Type0DPropTvoc> {
                let raw_value = self.get_tvoc_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the temperature value.
            pub fn get_temperature(&self) -> Option<Type0DPropTemperature> {
                let raw_value = self.get_temperature_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw CMF-T value.
            pub fn get_cmf_t_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(24, 1)
            }
            /// Get the CMF-T value.
            pub fn get_cmf_t(&self) -> Option<Type0DPropCmfT> {
                let raw_value = self.get_cmf_t_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw CMF-rh value.
            pub fn get_cmf_rh_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(25, 1)
            }
            /// Get the CMF-rh value.
            pub fn get_cmf_rh(&self) -> Option<Type0DPropCmfRh> {
                let raw_value = self.get_cmf_rh_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw unused value.
            pub fn get_unused_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(26, 1)
            }
            /// Get the unused value.
            pub fn get_unused(&self) -> Option<Type0DPropUnused> {
                let raw_value = self.get_unused_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw unused 1 value.
            pub fn get_unused_1_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(27, 1)
            }
            /// Get the unused 1 value.
            pub fn get_unused_1(&self) -> Option<Type0DPropUnused1> {
                let raw_value = self.get_unused_1_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw LrnBit value.
            pub fn get_lrnbit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LrnBit value.
            pub fn get_lrnbit(&self) -> Option<Type0DPropLrnbit> {
                let raw_value = self.get_lrnbit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw IAQ value.
            pub fn get_iaq_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(29, 3)
            }
            /// Get the IAQ value.
            pub fn get_iaq(&self) -> Option<Type0DPropIaq> {
                let raw_value = self.get_iaq_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type0D<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type0D")
                    .field("humidity", &self.get_humidity())
                    .field("tvoc", &self.get_tvoc())
                    .field("temperature", &self.get_temperature())
                    .field("cmf_t", &self.get_cmf_t())
                    .field("cmf_rh", &self.get_cmf_rh())
                    .field("unused", &self.get_unused())
                    .field("unused_1", &self.get_unused_1())
                    .field("lrnbit", &self.get_lrnbit())
                    .field("iaq", &self.get_iaq())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type0DPropHumidity {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type0DPropTvoc {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type0DPropTemperature {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0DPropCmfT {
            TemperatureOutOfComfortZone = false,
            TemperatureInComfortZone = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0DPropCmfRh {
            HumidityOutOfComfortZone = false,
            HumidityInComfortZone = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0DPropUnused {
            Always1 = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0DPropUnused1 {
            Always0 = false,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0DPropLrnbit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type0DPropIaq {
            ExcellentAirQualityTvoc010 = 0,
            GoodAirQualityTvoc10520 = 1,
            LightlyPollutedAirTvoc20530 = 2,
            ModeratelyPollutedAirTvoc30540 = 3,
            HeavilyPollutedAirTvoc40550 = 4,
            SeverelyPollutedAirTvoc50570 = 5,
            ExtremelyPollutedAirTvoc = 6,
            InvalidAirQuality = 7,
            _Other(u8),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func09<'b> {
        Type02(func09::Type02<'b>),
        Type04(func09::Type04<'b>),
        Type05(func09::Type05<'b>),
        Type06(func09::Type06<'b>),
        Type07(func09::Type07<'b>),
        Type08(func09::Type08<'b>),
        Type09(func09::Type09<'b>),
        Type0A(func09::Type0A<'b>),
        Type0B(func09::Type0B<'b>),
        Type0C(func09::Type0C<'b>),
        Type0D(func09::Type0D<'b>),
    }
    impl<'b> Func09<'b> {
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02(
                func09::Type02::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type04_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type04(
                func09::Type04::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type05_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type05(
                func09::Type05::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type06_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type06(
                func09::Type06::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type07_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type07(
                func09::Type07::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type08_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type08(
                func09::Type08::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type09_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type09(
                func09::Type09::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type0A_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type0A(
                func09::Type0A::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type0B_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type0B(
                func09::Type0B::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type0C_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type0C(
                func09::Type0C::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type0D_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type0D(
                func09::Type0D::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x04 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type04_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x05 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type05_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x06 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type06_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x07 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type07_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x08 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type08_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x09 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type09_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x0A => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type0A_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x0B => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type0B_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x0C => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type0C_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x0D => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type0D_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Controller Status (A5-11)
    pub mod func11 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Lighting Controller (A5-11-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type01PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Illumination value.
            pub fn get_illumination_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Illumination value in units of lx.
            pub fn get_illumination(&self) -> Option<f64> {
                let raw_value = self.get_illumination_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 510.0))
            }

            /// Get the raw Illumination Set Point value.
            pub fn get_illumination_set_point_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Illumination Set Point value in units of N/A.
            pub fn get_illumination_set_point(&self) -> Option<f64> {
                let raw_value = self.get_illumination_set_point_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 255.0))
            }

            /// Get the raw Dimming Output Level value.
            pub fn get_dimming_output_level_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Dimming Output Level value in units of N/A.
            pub fn get_dimming_output_level(&self) -> Option<f64> {
                let raw_value = self.get_dimming_output_level_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 255.0))
            }

            /// Get the raw Repeater value.
            pub fn get_repeater_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(24, 1)
            }
            /// Get the Repeater value.
            pub fn get_repeater(&self) -> Option<Type01PropRepeater> {
                let raw_value = self.get_repeater_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Power Relay Timer value.
            pub fn get_power_relay_timer_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(25, 1)
            }
            /// Get the Power Relay Timer value.
            pub fn get_power_relay_timer(&self) -> Option<Type01PropPowerRelayTimer> {
                let raw_value = self.get_power_relay_timer_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Daylight Harvesting value.
            pub fn get_daylight_harvesting_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(26, 1)
            }
            /// Get the Daylight Harvesting value.
            pub fn get_daylight_harvesting(&self) -> Option<Type01PropDaylightHarvesting> {
                let raw_value = self.get_daylight_harvesting_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Dimming value.
            pub fn get_dimming_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(27, 1)
            }
            /// Get the Dimming value.
            pub fn get_dimming(&self) -> Option<Type01PropDimming> {
                let raw_value = self.get_dimming_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Magnet Contact value.
            pub fn get_magnet_contact_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Magnet Contact value.
            pub fn get_magnet_contact(&self) -> Option<Type01PropMagnetContact> {
                let raw_value = self.get_magnet_contact_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Occupancy value.
            pub fn get_occupancy_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Occupancy value.
            pub fn get_occupancy(&self) -> Option<Type01PropOccupancy> {
                let raw_value = self.get_occupancy_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Power Relay value.
            pub fn get_power_relay_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Power Relay value.
            pub fn get_power_relay(&self) -> Option<Type01PropPowerRelay> {
                let raw_value = self.get_power_relay_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("illumination", &self.get_illumination())
                    .field("illumination_set_point", &self.get_illumination_set_point())
                    .field("dimming_output_level", &self.get_dimming_output_level())
                    .field("repeater", &self.get_repeater())
                    .field("power_relay_timer", &self.get_power_relay_timer())
                    .field("daylight_harvesting", &self.get_daylight_harvesting())
                    .field("dimming", &self.get_dimming())
                    .field("magnet_contact", &self.get_magnet_contact())
                    .field("occupancy", &self.get_occupancy())
                    .field("power_relay", &self.get_power_relay())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropRepeater {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropPowerRelayTimer {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropDaylightHarvesting {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropDimming {
            SwitchingLoad = false,
            DimmingLoad = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropMagnetContact {
            Open = false,
            Closed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropOccupancy {
            Unoccupied = false,
            Occupied = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropPowerRelay {
            Off = false,
            On = true,
            _Other(bool),
        }
        /// Temperature Controller Output (A5-11-02)
        #[derive(Clone, Copy)]
        pub struct Type02<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type02PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Control Variable value.
            pub fn get_control_variable_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Control Variable value in units of %.
            pub fn get_control_variable(&self) -> Option<f64> {
                let raw_value = self.get_control_variable_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 100.0))
            }

            /// Get the raw FanStage value.
            pub fn get_fanstage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the FanStage value.
            pub fn get_fanstage(&self) -> Option<Type02PropFanstage> {
                let raw_value = self.get_fanstage_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Actual Setpoint value.
            pub fn get_actual_setpoint_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Actual Setpoint value in units of °C.
            pub fn get_actual_setpoint(&self) -> Option<f64> {
                let raw_value = self.get_actual_setpoint_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 51.2))
            }

            /// Get the raw Alarm value.
            pub fn get_alarm_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(24, 1)
            }
            /// Get the Alarm value.
            pub fn get_alarm(&self) -> Option<Type02PropAlarm> {
                let raw_value = self.get_alarm_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Controller mode value.
            pub fn get_controller_mode_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(25, 2)
            }
            /// Get the Controller mode value.
            pub fn get_controller_mode(&self) -> Option<Type02PropControllerMode> {
                let raw_value = self.get_controller_mode_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Controller state value.
            pub fn get_controller_state_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(27, 1)
            }
            /// Get the Controller state value.
            pub fn get_controller_state(&self) -> Option<Type02PropControllerState> {
                let raw_value = self.get_controller_state_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy hold-off value.
            pub fn get_energy_hold_off_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Energy hold-off value.
            pub fn get_energy_hold_off(&self) -> Option<Type02PropEnergyHoldOff> {
                let raw_value = self.get_energy_hold_off_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Room occupancy value.
            pub fn get_room_occupancy_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(30, 2)
            }
            /// Get the Room occupancy value.
            pub fn get_room_occupancy(&self) -> Option<Type02PropRoomOccupancy> {
                let raw_value = self.get_room_occupancy_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("control_variable", &self.get_control_variable())
                    .field("fanstage", &self.get_fanstage())
                    .field("actual_setpoint", &self.get_actual_setpoint())
                    .field("alarm", &self.get_alarm())
                    .field("controller_mode", &self.get_controller_mode())
                    .field("controller_state", &self.get_controller_state())
                    .field("energy_hold_off", &self.get_energy_hold_off())
                    .field("room_occupancy", &self.get_room_occupancy())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type02PropFanstage {
            Stage0Manual = 0,
            Stage1Manual = 1,
            Stage2Manual = 2,
            Stage3Manual = 3,
            Stage0Automatic = 16,
            Stage1Automatic = 17,
            Stage2Automatic = 18,
            Stage3Automatic = 19,
            NotAvailable = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropAlarm {
            NoAlarm = false,
            Alarm = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type02PropControllerMode {
            Heating = 1,
            Cooling = 2,
            Off = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropControllerState {
            Automatic = false,
            Override = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropEnergyHoldOff {
            Normal = false,
            EnergyHoldOffDewPoint = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type02PropRoomOccupancy {
            Occupied = 0,
            Unoccupied = 1,
            Standby = 2,
            Frost = 3,
            _Other(u8),
        }
        /// Blind Status (A5-11-03)
        #[derive(Clone, Copy)]
        pub struct Type03<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type03<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type03PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Blind/shutter pos. value.
            pub fn get_blind_shutter_pos_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Blind/shutter pos. value in units of %.
            pub fn get_blind_shutter_pos(&self) -> Option<f64> {
                let raw_value = self.get_blind_shutter_pos_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 100.0, 0.0, 100.0))
            }

            /// Get the raw Angle sign value.
            pub fn get_angle_sign_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(8, 1)
            }
            /// Get the Angle sign value.
            pub fn get_angle_sign(&self) -> Option<Type03PropAngleSign> {
                let raw_value = self.get_angle_sign_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Angle value.
            pub fn get_angle_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(9, 7)
            }
            /// Get the Angle value in units of °.
            pub fn get_angle(&self) -> Option<f64> {
                let raw_value = self.get_angle_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 90.0, 0.0, 180.0))
            }

            /// Get the raw Position value flag value.
            pub fn get_position_value_flag_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(16, 1)
            }
            /// Get the Position value flag value.
            pub fn get_position_value_flag(&self) -> Option<Type03PropPositionValueFlag> {
                let raw_value = self.get_position_value_flag_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Angle value flag value.
            pub fn get_angle_value_flag_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(17, 1)
            }
            /// Get the Angle value flag value.
            pub fn get_angle_value_flag(&self) -> Option<Type03PropAngleValueFlag> {
                let raw_value = self.get_angle_value_flag_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Error state value.
            pub fn get_error_state_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(18, 2)
            }
            /// Get the Error state value.
            pub fn get_error_state(&self) -> Option<Type03PropErrorState> {
                let raw_value = self.get_error_state_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw End-position value.
            pub fn get_end_position_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(20, 2)
            }
            /// Get the End-position value.
            pub fn get_end_position(&self) -> Option<Type03PropEndPosition> {
                let raw_value = self.get_end_position_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Status value.
            pub fn get_status_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(22, 2)
            }
            /// Get the Status value.
            pub fn get_status(&self) -> Option<Type03PropStatus> {
                let raw_value = self.get_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Service Mode value.
            pub fn get_service_mode_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(24, 1)
            }
            /// Get the Service Mode value.
            pub fn get_service_mode(&self) -> Option<Type03PropServiceMode> {
                let raw_value = self.get_service_mode_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Mode of the position value.
            pub fn get_mode_of_the_position_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(25, 1)
            }
            /// Get the Mode of the position value.
            pub fn get_mode_of_the_position(&self) -> Option<Type03PropModeOfThePosition> {
                let raw_value = self.get_mode_of_the_position_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type03<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type03")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("blind_shutter_pos", &self.get_blind_shutter_pos())
                    .field("angle_sign", &self.get_angle_sign())
                    .field("angle", &self.get_angle())
                    .field("position_value_flag", &self.get_position_value_flag())
                    .field("angle_value_flag", &self.get_angle_value_flag())
                    .field("error_state", &self.get_error_state())
                    .field("end_position", &self.get_end_position())
                    .field("status", &self.get_status())
                    .field("service_mode", &self.get_service_mode())
                    .field("mode_of_the_position", &self.get_mode_of_the_position())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropAngleSign {
            PositiveSign = false,
            NegativeSign = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropPositionValueFlag {
            NoPositionValueAvailable = false,
            PositionValueAvailable = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropAngleValueFlag {
            NoAngleValueAvailable = false,
            AngleValueAvailable = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type03PropErrorState {
            NoErrorPresent = 0,
            EndPositionsAreNotConfigured = 1,
            InternalFailure = 2,
            NotUsed = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type03PropEndPosition {
            NoEndPositionAvailable = 0,
            NoEndPositionReached = 1,
            BlindFullyOpen = 2,
            BlindFullyClosed = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type03PropStatus {
            NoStatusAvailable = 0,
            BlindIsStopped = 1,
            BlindOpens = 2,
            BlindCloses = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropServiceMode {
            NormalMode = false,
            ServiceModeIsActivatedForExampleForMaintenance = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropModeOfThePosition {
            NormalMode = false,
            InverseMode = true,
            _Other(bool),
        }
        /// Extended Lighting Status (A5-11-04)
        #[derive(Clone, Copy)]
        pub struct Type04<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type04<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type04PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Parameter 1 value.
            pub fn get_parameter_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Parameter 1 value.
            pub fn get_parameter_1(&self) -> Option<Type04PropParameter1> {
                let raw_value = self.get_parameter_1_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Parameter 2 value.
            pub fn get_parameter_2_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Parameter 2 value.
            pub fn get_parameter_2(&self) -> Option<Type04PropParameter2> {
                let raw_value = self.get_parameter_2_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Parameter 3 value.
            pub fn get_parameter_3_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Parameter 3 value.
            pub fn get_parameter_3(&self) -> Option<Type04PropParameter3> {
                let raw_value = self.get_parameter_3_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Service Mode value.
            pub fn get_service_mode_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(24, 1)
            }
            /// Get the Service Mode value.
            pub fn get_service_mode(&self) -> Option<Type04PropServiceMode> {
                let raw_value = self.get_service_mode_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Operating hours flag value.
            pub fn get_operating_hours_flag_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(25, 1)
            }
            /// Get the Operating hours flag value.
            pub fn get_operating_hours_flag(&self) -> Option<Type04PropOperatingHoursFlag> {
                let raw_value = self.get_operating_hours_flag_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Error state value.
            pub fn get_error_state_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(26, 2)
            }
            /// Get the Error state value.
            pub fn get_error_state(&self) -> Option<Type04PropErrorState> {
                let raw_value = self.get_error_state_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Parameter Mode value.
            pub fn get_parameter_mode_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(29, 2)
            }
            /// Get the Parameter Mode value.
            pub fn get_parameter_mode(&self) -> Option<Type04PropParameterMode> {
                let raw_value = self.get_parameter_mode_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Status value.
            pub fn get_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Status value.
            pub fn get_status(&self) -> Option<Type04PropStatus> {
                let raw_value = self.get_status_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type04<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type04")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("parameter_1", &self.get_parameter_1())
                    .field("parameter_2", &self.get_parameter_2())
                    .field("parameter_3", &self.get_parameter_3())
                    .field("service_mode", &self.get_service_mode())
                    .field("operating_hours_flag", &self.get_operating_hours_flag())
                    .field("error_state", &self.get_error_state())
                    .field("parameter_mode", &self.get_parameter_mode())
                    .field("status", &self.get_status())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type04PropParameter1 {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type04PropParameter2 {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type04PropParameter3 {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropServiceMode {
            NormalMode = false,
            ServiceModeIsActivated = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropOperatingHoursFlag {
            NoLampOperatingHoursAvailable = false,
            LampOperatingHoursAvailable = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type04PropErrorState {
            NoErrorPresent = 0,
            LampFailure = 1,
            InternalFailure = 2,
            FailureOnTheExternalPeriphery = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type04PropParameterMode {
            _8BitDimmerValueAndLampOperatingHours = 0,
            RgbValue = 1,
            EnergyMeteringValue = 2,
            NotUsed = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropStatus {
            LightingOff = false,
            LightingOn = true,
            _Other(bool),
        }
        /// Dual-Channel Switch Actuator (A5-11-05), case 0
        #[derive(Clone, Copy)]
        pub struct Type05Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type05Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type05Case0PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Message Type value.
            pub fn get_message_type_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Message Type value.
            pub fn get_message_type(&self) -> Option<Type05Case0PropMessageType> {
                let raw_value = self.get_message_type_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type05Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type05Case0")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("message_type", &self.get_message_type())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case0PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case0PropMessageType {
            Request = false,
            _Other(bool),
        }
        /// Dual-Channel Switch Actuator (A5-11-05), case 1
        #[derive(Clone, Copy)]
        pub struct Type05Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type05Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type05Case1PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Working Mode value.
            pub fn get_working_mode_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(25, 3)
            }
            /// Get the Working Mode value.
            pub fn get_working_mode(&self) -> Option<Type05Case1PropWorkingMode> {
                let raw_value = self.get_working_mode_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Relay Status value.
            pub fn get_relay_status_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(29, 2)
            }
            /// Get the Relay Status value.
            pub fn get_relay_status(&self) -> Option<Type05Case1PropRelayStatus> {
                let raw_value = self.get_relay_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Message Type value.
            pub fn get_message_type_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Message Type value.
            pub fn get_message_type(&self) -> Option<Type05Case1PropMessageType> {
                let raw_value = self.get_message_type_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type05Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type05Case1")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("working_mode", &self.get_working_mode())
                    .field("relay_status", &self.get_relay_status())
                    .field("message_type", &self.get_message_type())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case1PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type05Case1PropWorkingMode {
            Mode1 = 1,
            Mode2 = 2,
            Mode3 = 3,
            Mode4 = 4,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type05Case1PropRelayStatus {
            Ch1OffCh2Off = 0,
            Ch1OnCh2Off = 1,
            Ch1OffCh2On = 2,
            Ch1OnCh2On = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case1PropMessageType {
            StatusReport = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func11<'b> {
        Type01(func11::Type01<'b>),
        Type02(func11::Type02<'b>),
        Type03(func11::Type03<'b>),
        Type04(func11::Type04<'b>),
        Type05Case0(func11::Type05Case0<'b>),
        Type05Case1(func11::Type05Case1<'b>),
    }
    impl<'b> Func11<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func11::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02(
                func11::Type02::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type03_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type03(
                func11::Type03::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type04_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type04(
                func11::Type04::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type05_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 2> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type05Case0(
                func11::Type05Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type05Case1(
                func11::Type05Case1::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 2>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x03 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type03_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x04 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type04_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x05 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type05_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Automated Meter Reading (AMR) (A5-12)
    pub mod func12 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Counter (A5-12-00)
        #[derive(Clone, Copy)]
        pub struct Type00<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type00PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter reading value.
            pub fn get_meter_reading_raw(&self) -> Option<u32> {
                self.reversed_bytes.u32_from_bits(0, 24)
            }

            /// Get the raw Measurement channel value.
            pub fn get_measurement_channel_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Measurement channel value in units of 1.
            pub fn get_measurement_channel(&self) -> Option<f64> {
                let raw_value = self.get_measurement_channel_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 15.0, 0.0, 15.0))
            }

            /// Get the raw Data type (unit) value.
            pub fn get_data_type_unit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Data type (unit) value.
            pub fn get_data_type_unit(&self) -> Option<Type00PropDataTypeUnit> {
                let raw_value = self.get_data_type_unit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Divisor (scale) value.
            pub fn get_divisor_scale_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(30, 2)
            }
            /// Get the Divisor (scale) value.
            pub fn get_divisor_scale(&self) -> Option<Type00PropDivisorScale> {
                let raw_value = self.get_divisor_scale_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("meter_reading", &self.get_meter_reading_raw())
                    .field("measurement_channel", &self.get_measurement_channel())
                    .field("data_type_unit", &self.get_data_type_unit())
                    .field("divisor_scale", &self.get_divisor_scale())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00PropDataTypeUnit {
            CumulativeValue = false,
            CurrentValue = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropDivisorScale {
            X1 = 0,
            X10 = 1,
            X100 = 2,
            X1000 = 3,
            _Other(u8),
        }
        /// Electricity (A5-12-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type01PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter reading value.
            pub fn get_meter_reading_raw(&self) -> Option<u32> {
                self.reversed_bytes.u32_from_bits(0, 24)
            }

            /// Get the raw Tariff info value.
            pub fn get_tariff_info_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Tariff info value in units of 1.
            pub fn get_tariff_info(&self) -> Option<f64> {
                let raw_value = self.get_tariff_info_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 15.0, 0.0, 15.0))
            }

            /// Get the raw Data type (unit) value.
            pub fn get_data_type_unit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Data type (unit) value.
            pub fn get_data_type_unit(&self) -> Option<Type01PropDataTypeUnit> {
                let raw_value = self.get_data_type_unit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Divisor (scale) value.
            pub fn get_divisor_scale_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(30, 2)
            }
            /// Get the Divisor (scale) value.
            pub fn get_divisor_scale(&self) -> Option<Type01PropDivisorScale> {
                let raw_value = self.get_divisor_scale_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("meter_reading", &self.get_meter_reading_raw())
                    .field("tariff_info", &self.get_tariff_info())
                    .field("data_type_unit", &self.get_data_type_unit())
                    .field("divisor_scale", &self.get_divisor_scale())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropDataTypeUnit {
            CumulativeValue = false,
            CurrentValue = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropDivisorScale {
            X1 = 0,
            X10 = 1,
            X100 = 2,
            X1000 = 3,
            _Other(u8),
        }
        /// Gas (A5-12-02)
        #[derive(Clone, Copy)]
        pub struct Type02<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type02PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw meter reading value.
            pub fn get_meter_reading_raw(&self) -> Option<u32> {
                self.reversed_bytes.u32_from_bits(0, 24)
            }

            /// Get the raw Tariff info value.
            pub fn get_tariff_info_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Tariff info value in units of 1.
            pub fn get_tariff_info(&self) -> Option<f64> {
                let raw_value = self.get_tariff_info_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 15.0, 0.0, 15.0))
            }

            /// Get the raw data type (unit) value.
            pub fn get_data_type_unit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the data type (unit) value.
            pub fn get_data_type_unit(&self) -> Option<Type02PropDataTypeUnit> {
                let raw_value = self.get_data_type_unit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw divisor (scale) value.
            pub fn get_divisor_scale_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(30, 2)
            }
            /// Get the divisor (scale) value.
            pub fn get_divisor_scale(&self) -> Option<Type02PropDivisorScale> {
                let raw_value = self.get_divisor_scale_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("meter_reading", &self.get_meter_reading_raw())
                    .field("tariff_info", &self.get_tariff_info())
                    .field("data_type_unit", &self.get_data_type_unit())
                    .field("divisor_scale", &self.get_divisor_scale())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropDataTypeUnit {
            CumulativeValue = false,
            CurrentValue = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type02PropDivisorScale {
            X1 = 0,
            X10 = 1,
            X100 = 2,
            X1000 = 3,
            _Other(u8),
        }
        /// Water (A5-12-03)
        #[derive(Clone, Copy)]
        pub struct Type03<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type03<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type03PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter reading value.
            pub fn get_meter_reading_raw(&self) -> Option<u32> {
                self.reversed_bytes.u32_from_bits(0, 24)
            }

            /// Get the raw Tariff info value.
            pub fn get_tariff_info_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Tariff info value in units of 1.
            pub fn get_tariff_info(&self) -> Option<f64> {
                let raw_value = self.get_tariff_info_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 15.0, 0.0, 15.0))
            }

            /// Get the raw Data type (unit) value.
            pub fn get_data_type_unit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Data type (unit) value.
            pub fn get_data_type_unit(&self) -> Option<Type03PropDataTypeUnit> {
                let raw_value = self.get_data_type_unit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Divisor (scale) value.
            pub fn get_divisor_scale_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(30, 2)
            }
            /// Get the Divisor (scale) value.
            pub fn get_divisor_scale(&self) -> Option<Type03PropDivisorScale> {
                let raw_value = self.get_divisor_scale_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type03<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type03")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("meter_reading", &self.get_meter_reading_raw())
                    .field("tariff_info", &self.get_tariff_info())
                    .field("data_type_unit", &self.get_data_type_unit())
                    .field("divisor_scale", &self.get_divisor_scale())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropDataTypeUnit {
            CumulativeValue = false,
            CurrentValue = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type03PropDivisorScale {
            X1 = 0,
            X10 = 1,
            X100 = 2,
            X1000 = 3,
            _Other(u8),
        }
        /// Temperature and Load Sensor (A5-12-04)
        #[derive(Clone, Copy)]
        pub struct Type04<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type04<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type04PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter reading value.
            pub fn get_meter_reading_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(0, 14)
            }
            /// Get the Meter reading value in units of gram.
            pub fn get_meter_reading(&self) -> Option<f64> {
                let raw_value = self.get_meter_reading_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 16383.0, 0.0, 16383.0))
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, -40.0, 40.0))
            }

            /// Get the raw Battery Level value.
            pub fn get_battery_level_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(30, 2)
            }
            /// Get the Battery Level value.
            pub fn get_battery_level(&self) -> Option<Type04PropBatteryLevel> {
                let raw_value = self.get_battery_level_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type04<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type04")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("meter_reading", &self.get_meter_reading())
                    .field("temperature", &self.get_temperature())
                    .field("battery_level", &self.get_battery_level())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type04PropBatteryLevel {
            _100Minus75 = 0,
            _75Minus50 = 1,
            _50Minus25 = 2,
            _25Minus0 = 3,
            _Other(u8),
        }
        /// Temperature and Container Sensor (A5-12-05)
        #[derive(Clone, Copy)]
        pub struct Type05<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type05<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type05PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Position Sensor 0 value.
            pub fn get_position_sensor_0_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(0, 1)
            }
            /// Get the Position Sensor 0 value.
            pub fn get_position_sensor_0(&self) -> Option<Type05PropPositionSensor0> {
                let raw_value = self.get_position_sensor_0_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Position Sensor 1 value.
            pub fn get_position_sensor_1_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(1, 1)
            }
            /// Get the Position Sensor 1 value.
            pub fn get_position_sensor_1(&self) -> Option<Type05PropPositionSensor1> {
                let raw_value = self.get_position_sensor_1_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Position Sensor 2 value.
            pub fn get_position_sensor_2_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(2, 1)
            }
            /// Get the Position Sensor 2 value.
            pub fn get_position_sensor_2(&self) -> Option<Type05PropPositionSensor2> {
                let raw_value = self.get_position_sensor_2_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Position Sensor 3 value.
            pub fn get_position_sensor_3_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(3, 1)
            }
            /// Get the Position Sensor 3 value.
            pub fn get_position_sensor_3(&self) -> Option<Type05PropPositionSensor3> {
                let raw_value = self.get_position_sensor_3_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Position Sensor 4 value.
            pub fn get_position_sensor_4_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(4, 1)
            }
            /// Get the Position Sensor 4 value.
            pub fn get_position_sensor_4(&self) -> Option<Type05PropPositionSensor4> {
                let raw_value = self.get_position_sensor_4_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Position Sensor 5 value.
            pub fn get_position_sensor_5_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(5, 1)
            }
            /// Get the Position Sensor 5 value.
            pub fn get_position_sensor_5(&self) -> Option<Type05PropPositionSensor5> {
                let raw_value = self.get_position_sensor_5_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Position Sensor 6 value.
            pub fn get_position_sensor_6_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(6, 1)
            }
            /// Get the Position Sensor 6 value.
            pub fn get_position_sensor_6(&self) -> Option<Type05PropPositionSensor6> {
                let raw_value = self.get_position_sensor_6_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Position Sensor 7 value.
            pub fn get_position_sensor_7_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(7, 1)
            }
            /// Get the Position Sensor 7 value.
            pub fn get_position_sensor_7(&self) -> Option<Type05PropPositionSensor7> {
                let raw_value = self.get_position_sensor_7_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Position Sensor 8 value.
            pub fn get_position_sensor_8_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(8, 1)
            }
            /// Get the Position Sensor 8 value.
            pub fn get_position_sensor_8(&self) -> Option<Type05PropPositionSensor8> {
                let raw_value = self.get_position_sensor_8_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Position Sensor 9 value.
            pub fn get_position_sensor_9_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(9, 1)
            }
            /// Get the Position Sensor 9 value.
            pub fn get_position_sensor_9(&self) -> Option<Type05PropPositionSensor9> {
                let raw_value = self.get_position_sensor_9_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, -40.0, 40.0))
            }

            /// Get the raw Battery Level value.
            pub fn get_battery_level_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(30, 2)
            }
            /// Get the Battery Level value.
            pub fn get_battery_level(&self) -> Option<Type05PropBatteryLevel> {
                let raw_value = self.get_battery_level_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type05<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type05")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("position_sensor_0", &self.get_position_sensor_0())
                    .field("position_sensor_1", &self.get_position_sensor_1())
                    .field("position_sensor_2", &self.get_position_sensor_2())
                    .field("position_sensor_3", &self.get_position_sensor_3())
                    .field("position_sensor_4", &self.get_position_sensor_4())
                    .field("position_sensor_5", &self.get_position_sensor_5())
                    .field("position_sensor_6", &self.get_position_sensor_6())
                    .field("position_sensor_7", &self.get_position_sensor_7())
                    .field("position_sensor_8", &self.get_position_sensor_8())
                    .field("position_sensor_9", &self.get_position_sensor_9())
                    .field("temperature", &self.get_temperature())
                    .field("battery_level", &self.get_battery_level())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropPositionSensor0 {
            NotPossessed = false,
            Possessed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropPositionSensor1 {
            NotPossessed = false,
            Possessed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropPositionSensor2 {
            NotPossessed = false,
            Possessed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropPositionSensor3 {
            NotPossessed = false,
            Possessed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropPositionSensor4 {
            NotPossessed = false,
            Possessed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropPositionSensor5 {
            NotPossessed = false,
            Possessed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropPositionSensor6 {
            NotPossessed = false,
            Possessed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropPositionSensor7 {
            NotPossessed = false,
            Possessed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropPositionSensor8 {
            NotPossessed = false,
            Possessed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropPositionSensor9 {
            NotPossessed = false,
            Possessed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type05PropBatteryLevel {
            _100Minus75 = 0,
            _75Minus50 = 1,
            _50Minus25 = 2,
            _25Minus0 = 3,
            _Other(u8),
        }
        /// Current meter 16 channels (A5-12-10)
        #[derive(Clone, Copy)]
        pub struct Type10<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type10<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type10PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter reading value.
            pub fn get_meter_reading_raw(&self) -> Option<u32> {
                self.reversed_bytes.u32_from_bits(0, 24)
            }

            /// Get the raw Measurement channel value.
            pub fn get_measurement_channel_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Measurement channel value.
            pub fn get_measurement_channel(&self) -> Option<f64> {
                let raw_value = self.get_measurement_channel_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 15.0, 0.0, 15.0))
            }

            /// Get the raw Data type (unit) value.
            pub fn get_data_type_unit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Data type (unit) value.
            pub fn get_data_type_unit(&self) -> Option<Type10PropDataTypeUnit> {
                let raw_value = self.get_data_type_unit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Divisor (scale) value.
            pub fn get_divisor_scale_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(30, 2)
            }
            /// Get the Divisor (scale) value.
            pub fn get_divisor_scale(&self) -> Option<Type10PropDivisorScale> {
                let raw_value = self.get_divisor_scale_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type10<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type10")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("meter_reading", &self.get_meter_reading_raw())
                    .field("measurement_channel", &self.get_measurement_channel())
                    .field("data_type_unit", &self.get_data_type_unit())
                    .field("divisor_scale", &self.get_divisor_scale())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type10PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type10PropDataTypeUnit {
            CumulativeValue = false,
            CurrentValue = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type10PropDivisorScale {
            X1 = 0,
            X10 = 1,
            X100 = 2,
            X1000 = 3,
            _Other(u8),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func12<'b> {
        Type00(func12::Type00<'b>),
        Type01(func12::Type01<'b>),
        Type02(func12::Type02<'b>),
        Type03(func12::Type03<'b>),
        Type04(func12::Type04<'b>),
        Type05(func12::Type05<'b>),
        Type10(func12::Type10<'b>),
    }
    impl<'b> Func12<'b> {
        pub fn type00_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type00(
                func12::Type00::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func12::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02(
                func12::Type02::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type03_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type03(
                func12::Type03::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type04_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type04(
                func12::Type04::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type05_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type05(
                func12::Type05::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type10_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type10(
                func12::Type10::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x00 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type00_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x03 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type03_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x04 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type04_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x05 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type05_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x10 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type10_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Environmental Applications (A5-13)
    pub mod func13 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Weather Station (A5-13-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type01PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Dawn sensor value.
            pub fn get_dawn_sensor_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Dawn sensor value in units of lx.
            pub fn get_dawn_sensor(&self) -> Option<f64> {
                let raw_value = self.get_dawn_sensor_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 999.0))
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, -40.0, 80.0))
            }

            /// Get the raw Wind speed value.
            pub fn get_wind_speed_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Wind speed value in units of m/s.
            pub fn get_wind_speed(&self) -> Option<f64> {
                let raw_value = self.get_wind_speed_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 70.0))
            }

            /// Get the raw Identifier value.
            pub fn get_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Identifier value.
            pub fn get_identifier(&self) -> Option<Type01PropIdentifier> {
                let raw_value = self.get_identifier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Day / Night value.
            pub fn get_day_night_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Day / Night value.
            pub fn get_day_night(&self) -> Option<Type01PropDayNight> {
                let raw_value = self.get_day_night_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Rain Indication value.
            pub fn get_rain_indication_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Rain Indication value.
            pub fn get_rain_indication(&self) -> Option<Type01PropRainIndication> {
                let raw_value = self.get_rain_indication_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("dawn_sensor", &self.get_dawn_sensor())
                    .field("temperature", &self.get_temperature())
                    .field("wind_speed", &self.get_wind_speed())
                    .field("identifier", &self.get_identifier())
                    .field("day_night", &self.get_day_night())
                    .field("rain_indication", &self.get_rain_indication())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropIdentifier {
            Value1 = 1,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropDayNight {
            Day = false,
            Night = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropRainIndication {
            NoRain = false,
            Rain = true,
            _Other(bool),
        }
        /// Sun Intensity (A5-13-02)
        #[derive(Clone, Copy)]
        pub struct Type02<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type02PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Sun – West value.
            pub fn get_sun_west_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Sun – West value in units of klx.
            pub fn get_sun_west(&self) -> Option<f64> {
                let raw_value = self.get_sun_west_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 150.0))
            }

            /// Get the raw Sun – South value.
            pub fn get_sun_south_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Sun – South value in units of klx.
            pub fn get_sun_south(&self) -> Option<f64> {
                let raw_value = self.get_sun_south_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 150.0))
            }

            /// Get the raw Sun – East value.
            pub fn get_sun_east_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Sun – East value in units of klx.
            pub fn get_sun_east(&self) -> Option<f64> {
                let raw_value = self.get_sun_east_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 150.0))
            }

            /// Get the raw Identifier value.
            pub fn get_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Identifier value.
            pub fn get_identifier(&self) -> Option<Type02PropIdentifier> {
                let raw_value = self.get_identifier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Hemisphere value.
            pub fn get_hemisphere_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Hemisphere value.
            pub fn get_hemisphere(&self) -> Option<Type02PropHemisphere> {
                let raw_value = self.get_hemisphere_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("sun_west", &self.get_sun_west())
                    .field("sun_south", &self.get_sun_south())
                    .field("sun_east", &self.get_sun_east())
                    .field("identifier", &self.get_identifier())
                    .field("hemisphere", &self.get_hemisphere())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type02PropIdentifier {
            Value2 = 2,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropHemisphere {
            North = false,
            South = true,
            _Other(bool),
        }
        /// Date Exchange (A5-13-03)
        #[derive(Clone, Copy)]
        pub struct Type03<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type03<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type03PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Day value.
            pub fn get_day_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(3, 5)
            }
            /// Get the Day value in units of N/A.
            pub fn get_day(&self) -> Option<f64> {
                let raw_value = self.get_day_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 31.0, 1.0, 31.0))
            }

            /// Get the raw Month value.
            pub fn get_month_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(12, 4)
            }
            /// Get the Month value in units of N/A.
            pub fn get_month(&self) -> Option<f64> {
                let raw_value = self.get_month_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 12.0, 1.0, 12.0))
            }

            /// Get the raw Year value.
            pub fn get_year_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(17, 7)
            }
            /// Get the Year value in units of N/A.
            pub fn get_year(&self) -> Option<f64> {
                let raw_value = self.get_year_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 99.0, 2000.0, 2099.0))
            }

            /// Get the raw Identifier value.
            pub fn get_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Identifier value.
            pub fn get_identifier(&self) -> Option<Type03PropIdentifier> {
                let raw_value = self.get_identifier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Source value.
            pub fn get_source_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Source value.
            pub fn get_source(&self) -> Option<Type03PropSource> {
                let raw_value = self.get_source_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type03<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type03")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("day", &self.get_day())
                    .field("month", &self.get_month())
                    .field("year", &self.get_year())
                    .field("identifier", &self.get_identifier())
                    .field("source", &self.get_source())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type03PropIdentifier {
            Value3 = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropSource {
            RealTimeClock = false,
            GpsOrEquivalentEGDcf77Wwv = true,
            _Other(bool),
        }
        /// Time and Day Exchange (A5-13-04)
        #[derive(Clone, Copy)]
        pub struct Type04<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type04<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type04PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Weekday value.
            pub fn get_weekday_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 3)
            }
            /// Get the Weekday value.
            pub fn get_weekday(&self) -> Option<Type04PropWeekday> {
                let raw_value = self.get_weekday_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Hour value.
            pub fn get_hour_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(3, 5)
            }
            /// Get the Hour value in units of N/A.
            pub fn get_hour(&self) -> Option<f64> {
                let raw_value = self.get_hour_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 23.0, 0.0, 23.0))
            }

            /// Get the raw Minute value.
            pub fn get_minute_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(10, 6)
            }
            /// Get the Minute value in units of N/A.
            pub fn get_minute(&self) -> Option<f64> {
                let raw_value = self.get_minute_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 59.0, 0.0, 59.0))
            }

            /// Get the raw Second value.
            pub fn get_second_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(18, 6)
            }
            /// Get the Second value in units of N/A.
            pub fn get_second(&self) -> Option<f64> {
                let raw_value = self.get_second_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 59.0, 0.0, 59.0))
            }

            /// Get the raw Identifier value.
            pub fn get_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Identifier value.
            pub fn get_identifier(&self) -> Option<Type04PropIdentifier> {
                let raw_value = self.get_identifier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Time Format value.
            pub fn get_time_format_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Time Format value.
            pub fn get_time_format(&self) -> Option<Type04PropTimeFormat> {
                let raw_value = self.get_time_format_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw AM/PM value.
            pub fn get_am_pm_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the AM/PM value.
            pub fn get_am_pm(&self) -> Option<Type04PropAmPm> {
                let raw_value = self.get_am_pm_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Source value.
            pub fn get_source_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Source value.
            pub fn get_source(&self) -> Option<Type04PropSource> {
                let raw_value = self.get_source_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type04<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type04")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("weekday", &self.get_weekday())
                    .field("hour", &self.get_hour())
                    .field("minute", &self.get_minute())
                    .field("second", &self.get_second())
                    .field("identifier", &self.get_identifier())
                    .field("time_format", &self.get_time_format())
                    .field("am_pm", &self.get_am_pm())
                    .field("source", &self.get_source())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type04PropWeekday {
            Monday = 1,
            Tuesday = 2,
            Wednesday = 3,
            Thursday = 4,
            Friday = 5,
            Saturday = 6,
            Sunday = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type04PropIdentifier {
            Value4 = 4,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropTimeFormat {
            _24Hours = false,
            _12Hours = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropAmPm {
            Am = false,
            Pm = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropSource {
            RealTimeClock = false,
            GpsOrEquivalentEGDcf77Wwv = true,
            _Other(bool),
        }
        /// Direction Exchange (A5-13-05)
        #[derive(Clone, Copy)]
        pub struct Type05<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type05<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type05PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Elevation value.
            pub fn get_elevation_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Elevation value in units of °.
            pub fn get_elevation(&self) -> Option<f64> {
                let raw_value = self.get_elevation_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 180.0, -90.0, 90.0))
            }

            /// Get the raw Azimut value.
            pub fn get_azimut_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(15, 9)
            }
            /// Get the Azimut value in units of °.
            pub fn get_azimut(&self) -> Option<f64> {
                let raw_value = self.get_azimut_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 359.0, 0.0, 359.0))
            }

            /// Get the raw Identifier value.
            pub fn get_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Identifier value.
            pub fn get_identifier(&self) -> Option<Type05PropIdentifier> {
                let raw_value = self.get_identifier_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type05<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type05")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("elevation", &self.get_elevation())
                    .field("azimut", &self.get_azimut())
                    .field("identifier", &self.get_identifier())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type05PropIdentifier {
            Value5 = 5,
            _Other(u8),
        }
        /// Geographic Position Exchange (A5-13-06)
        #[derive(Clone, Copy)]
        pub struct Type06<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type06<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type06PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Latitude(MSB) value.
            pub fn get_latitude_msb_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }

            /// Get the raw Longitude(MSB) value.
            pub fn get_longitude_msb_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }

            /// Get the raw Latitude(LSB) value.
            pub fn get_latitude_lsb_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Latitude(LSB) value in units of °.
            pub fn get_latitude_lsb(&self) -> Option<f64> {
                let raw_value = self.get_latitude_lsb_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 4095.0, -90.0, 90.0))
            }

            /// Get the raw Longitude(LSB) value.
            pub fn get_longitude_lsb_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Longitude(LSB) value in units of °.
            pub fn get_longitude_lsb(&self) -> Option<f64> {
                let raw_value = self.get_longitude_lsb_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 4095.0, -180.0, 180.0))
            }

            /// Get the raw Identifier value.
            pub fn get_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Identifier value.
            pub fn get_identifier(&self) -> Option<Type06PropIdentifier> {
                let raw_value = self.get_identifier_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type06<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type06")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("latitude_msb", &self.get_latitude_msb_raw())
                    .field("longitude_msb", &self.get_longitude_msb_raw())
                    .field("latitude_lsb", &self.get_latitude_lsb())
                    .field("longitude_lsb", &self.get_longitude_lsb())
                    .field("identifier", &self.get_identifier())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type06PropIdentifier {
            Value6 = 6,
            _Other(u8),
        }
        /// Wind Sensor (A5-13-07)
        #[derive(Clone, Copy)]
        pub struct Type07<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type07<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type07PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Wind Direction value.
            pub fn get_wind_direction_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Wind Direction value.
            pub fn get_wind_direction(&self) -> Option<Type07PropWindDirection> {
                let raw_value = self.get_wind_direction_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Average Wind Speed value.
            pub fn get_average_wind_speed_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Average Wind Speed value in units of mph.
            pub fn get_average_wind_speed(&self) -> Option<f64> {
                let raw_value = self.get_average_wind_speed_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 1.0, 199.9))
            }

            /// Get the raw Maximum Wind Speed value.
            pub fn get_maximum_wind_speed_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Maximum Wind Speed value in units of mph.
            pub fn get_maximum_wind_speed(&self) -> Option<f64> {
                let raw_value = self.get_maximum_wind_speed_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 1.0, 199.9))
            }

            /// Get the raw Battery Status value.
            pub fn get_battery_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Battery Status value.
            pub fn get_battery_status(&self) -> Option<Type07PropBatteryStatus> {
                let raw_value = self.get_battery_status_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type07<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type07")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("wind_direction", &self.get_wind_direction())
                    .field("average_wind_speed", &self.get_average_wind_speed())
                    .field("maximum_wind_speed", &self.get_maximum_wind_speed())
                    .field("battery_status", &self.get_battery_status())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type07PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type07PropWindDirection {
            Nne = 0,
            Ne = 1,
            Ene = 2,
            E = 3,
            Ese = 4,
            Se = 5,
            Sse = 6,
            S = 7,
            Ssw = 8,
            Sw = 9,
            Wsw = 10,
            W = 11,
            Wnw = 12,
            Nw = 13,
            Nnw = 14,
            N = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type07PropBatteryStatus {
            BatteryOkay = false,
            BatteryLow = true,
            _Other(bool),
        }
        /// Rain Sensor (A5-13-08)
        #[derive(Clone, Copy)]
        pub struct Type08<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type08<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type08PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Rainfall Adjust Sign value.
            pub fn get_rainfall_adjust_sign_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(1, 1)
            }
            /// Get the Rainfall Adjust Sign value.
            pub fn get_rainfall_adjust_sign(&self) -> Option<Type08PropRainfallAdjustSign> {
                let raw_value = self.get_rainfall_adjust_sign_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Rainfall Adjust value.
            pub fn get_rainfall_adjust_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(2, 6)
            }
            /// Get the Rainfall Adjust value.
            pub fn get_rainfall_adjust(&self) -> Option<Type08PropRainfallAdjust> {
                let raw_value = self.get_rainfall_adjust_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Rainfall Count value.
            pub fn get_rainfall_count_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(8, 16)
            }
            /// Get the Rainfall Count value.
            pub fn get_rainfall_count(&self) -> Option<f64> {
                let raw_value = self.get_rainfall_count_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 65535.0, 0.0, 65535.0))
            }

            /// Get the raw Battery Status value.
            pub fn get_battery_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Battery Status value.
            pub fn get_battery_status(&self) -> Option<Type08PropBatteryStatus> {
                let raw_value = self.get_battery_status_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type08<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type08")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("rainfall_adjust_sign", &self.get_rainfall_adjust_sign())
                    .field("rainfall_adjust", &self.get_rainfall_adjust())
                    .field("rainfall_count", &self.get_rainfall_count())
                    .field("battery_status", &self.get_battery_status())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08PropRainfallAdjustSign {
            Negative = false,
            Positive = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type08PropRainfallAdjust {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08PropBatteryStatus {
            BatteryOkay = false,
            BatteryLow = true,
            _Other(bool),
        }
        /// Sun position and radiation (A5-13-10)
        #[derive(Clone, Copy)]
        pub struct Type10<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type10<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type10PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Day / Night value.
            pub fn get_day_night_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(7, 1)
            }
            /// Get the Day / Night value.
            pub fn get_day_night(&self) -> Option<Type10PropDayNight> {
                let raw_value = self.get_day_night_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Sun Elevation value.
            pub fn get_sun_elevation_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 7)
            }
            /// Get the Sun Elevation value in units of °.
            pub fn get_sun_elevation(&self) -> Option<f64> {
                let raw_value = self.get_sun_elevation_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 90.0, 0.0, 90.0))
            }

            /// Get the raw Sun Azimuth value.
            pub fn get_sun_azimuth_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Sun Azimuth value in units of °.
            pub fn get_sun_azimuth(&self) -> Option<f64> {
                let raw_value = self.get_sun_azimuth_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 180.0, -90.0, 90.0))
            }

            /// Get the raw Solar Radiation (MSB) value.
            pub fn get_solar_radiation_msb_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }

            /// Get the raw Solar Radiation (LSB) value.
            pub fn get_solar_radiation_lsb_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(29, 3)
            }
            /// Get the Solar Radiation (LSB) value in units of W/m2.
            pub fn get_solar_radiation_lsb(&self) -> Option<f64> {
                let raw_value = self.get_solar_radiation_lsb_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 2000.0, 0.0, 2000.0))
            }

            /// Get the raw Identifier value.
            pub fn get_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Identifier value.
            pub fn get_identifier(&self) -> Option<Type10PropIdentifier> {
                let raw_value = self.get_identifier_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type10<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type10")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("day_night", &self.get_day_night())
                    .field("sun_elevation", &self.get_sun_elevation())
                    .field("sun_azimuth", &self.get_sun_azimuth())
                    .field("solar_radiation_msb", &self.get_solar_radiation_msb_raw())
                    .field("solar_radiation_lsb", &self.get_solar_radiation_lsb())
                    .field("identifier", &self.get_identifier())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type10PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type10PropDayNight {
            Day = false,
            Night = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type10PropIdentifier {
            Value7 = 7,
            _Other(u8),
        }
        /// Noise (A5-13-11)
        #[derive(Clone, Copy)]
        pub struct Type11<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type11<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Average sound level value.
            pub fn get_average_sound_level_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(0, 10)
            }
            /// Get the Average sound level value in units of 0.1 dBA.
            pub fn get_average_sound_level(&self) -> Option<f64> {
                let raw_value = self.get_average_sound_level_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 1023.0, 17.6, 120.0))
            }

            /// Get the raw Pic sound level value.
            pub fn get_pic_sound_level_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(10, 10)
            }
            /// Get the Pic sound level value in units of 0.1 dBA.
            pub fn get_pic_sound_level(&self) -> Option<f64> {
                let raw_value = self.get_pic_sound_level_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 1023.0, 17.6, 120.0))
            }

            /// Get the raw Source localization value.
            pub fn get_source_localization_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(20, 8)
            }
            /// Get the Source localization value.
            pub fn get_source_localization(&self) -> Option<Type11PropSourceLocalization> {
                let raw_value = self.get_source_localization_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw LrnBit value.
            pub fn get_lrnbit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LrnBit value.
            pub fn get_lrnbit(&self) -> Option<Type11PropLrnbit> {
                let raw_value = self.get_lrnbit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Localization active value.
            pub fn get_localization_active_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Localization active value.
            pub fn get_localization_active(&self) -> Option<Type11PropLocalizationActive> {
                let raw_value = self.get_localization_active_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type11<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type11")
                    .field("average_sound_level", &self.get_average_sound_level())
                    .field("pic_sound_level", &self.get_pic_sound_level())
                    .field("source_localization", &self.get_source_localization())
                    .field("lrnbit", &self.get_lrnbit())
                    .field("localization_active", &self.get_localization_active())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type11PropSourceLocalization {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type11PropLrnbit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type11PropLocalizationActive {
            NoLocalization = false,
            LocalizationActive = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func13<'b> {
        Type01(func13::Type01<'b>),
        Type02(func13::Type02<'b>),
        Type03(func13::Type03<'b>),
        Type04(func13::Type04<'b>),
        Type05(func13::Type05<'b>),
        Type06(func13::Type06<'b>),
        Type07(func13::Type07<'b>),
        Type08(func13::Type08<'b>),
        Type10(func13::Type10<'b>),
        Type11(func13::Type11<'b>),
    }
    impl<'b> Func13<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func13::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02(
                func13::Type02::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type03_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type03(
                func13::Type03::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type04_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type04(
                func13::Type04::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type05_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type05(
                func13::Type05::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type06_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type06(
                func13::Type06::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type07_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type07(
                func13::Type07::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type08_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type08(
                func13::Type08::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type10_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type10(
                func13::Type10::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type11_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type11(
                func13::Type11::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x03 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type03_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x04 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type04_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x05 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type05_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x06 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type06_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x07 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type07_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x08 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type08_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x10 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type10_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x11 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type11_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Multi-Func Sensor (A5-14)
    pub mod func14 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Single Input Contact (Window/Door), Supply voltage monitor (A5-14-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type01PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 5.0))
            }

            /// Get the raw Contact value.
            pub fn get_contact_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Contact value.
            pub fn get_contact(&self) -> Option<Type01PropContact> {
                let raw_value = self.get_contact_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("contact", &self.get_contact())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropContact {
            ContactClosed = false,
            ContactOpen = true,
            _Other(bool),
        }
        /// Single Input Contact (Window/Door), Supply voltage monitor and Illumination (A5-14-02)
        #[derive(Clone, Copy)]
        pub struct Type02<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type02PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 5.0))
            }

            /// Get the raw Illumination value.
            pub fn get_illumination_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Illumination value in units of lx.
            pub fn get_illumination(&self) -> Option<f64> {
                let raw_value = self.get_illumination_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 1000.0))
            }

            /// Get the raw Contact value.
            pub fn get_contact_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Contact value.
            pub fn get_contact(&self) -> Option<Type02PropContact> {
                let raw_value = self.get_contact_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("illumination", &self.get_illumination())
                    .field("contact", &self.get_contact())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropContact {
            ContactClosed = false,
            ContactOpen = true,
            _Other(bool),
        }
        /// Single Input Contact (Window/Door), Supply voltage monitor and Vibration (A5-14-03)
        #[derive(Clone, Copy)]
        pub struct Type03<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type03<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type03PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 5.0))
            }

            /// Get the raw Vibration value.
            pub fn get_vibration_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Vibration value.
            pub fn get_vibration(&self) -> Option<Type03PropVibration> {
                let raw_value = self.get_vibration_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Contact value.
            pub fn get_contact_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Contact value.
            pub fn get_contact(&self) -> Option<Type03PropContact> {
                let raw_value = self.get_contact_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type03<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type03")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("vibration", &self.get_vibration())
                    .field("contact", &self.get_contact())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropVibration {
            NoVibrationDetected = false,
            VibrationDetected = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropContact {
            ContactClosed = false,
            ContactOpen = true,
            _Other(bool),
        }
        /// Single Input Contact (Window/Door), Supply voltage monitor, Vibration and Illumination (A5-14-04)
        #[derive(Clone, Copy)]
        pub struct Type04<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type04<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type04PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 5.0))
            }

            /// Get the raw Illumination value.
            pub fn get_illumination_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Illumination value in units of lx.
            pub fn get_illumination(&self) -> Option<f64> {
                let raw_value = self.get_illumination_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 1000.0))
            }

            /// Get the raw Vibration value.
            pub fn get_vibration_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Vibration value.
            pub fn get_vibration(&self) -> Option<Type04PropVibration> {
                let raw_value = self.get_vibration_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Contact value.
            pub fn get_contact_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Contact value.
            pub fn get_contact(&self) -> Option<Type04PropContact> {
                let raw_value = self.get_contact_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type04<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type04")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("illumination", &self.get_illumination())
                    .field("vibration", &self.get_vibration())
                    .field("contact", &self.get_contact())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropVibration {
            NoVibrationDetected = false,
            VibrationDetected = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropContact {
            ContactClosed = false,
            ContactOpen = true,
            _Other(bool),
        }
        /// Vibration/Tilt, Supply voltage monitor (A5-14-05)
        #[derive(Clone, Copy)]
        pub struct Type05<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type05<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type05PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 5.0))
            }

            /// Get the raw Vibration value.
            pub fn get_vibration_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Vibration value.
            pub fn get_vibration(&self) -> Option<Type05PropVibration> {
                let raw_value = self.get_vibration_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type05<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type05")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("vibration", &self.get_vibration())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropVibration {
            NoVibrationDetected = false,
            VibrationDetected = true,
            _Other(bool),
        }
        /// Vibration/Tilt, Illumination and Supply voltage monitor (A5-14-06)
        #[derive(Clone, Copy)]
        pub struct Type06<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type06<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type06PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 5.0))
            }

            /// Get the raw Illumination value.
            pub fn get_illumination_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Illumination value in units of lx.
            pub fn get_illumination(&self) -> Option<f64> {
                let raw_value = self.get_illumination_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 1000.0))
            }

            /// Get the raw Vibration value.
            pub fn get_vibration_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Vibration value.
            pub fn get_vibration(&self) -> Option<Type06PropVibration> {
                let raw_value = self.get_vibration_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type06<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type06")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("illumination", &self.get_illumination())
                    .field("vibration", &self.get_vibration())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06PropVibration {
            NoVibrationDetected = false,
            VibrationDetected = true,
            _Other(bool),
        }
        /// Dual-door-contact with States Open/Closed and Locked/Unlocked, Supply voltage monitor (A5-14-07)
        #[derive(Clone, Copy)]
        pub struct Type07<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type07<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type07PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 5.0))
            }

            /// Get the raw Door Contact value.
            pub fn get_door_contact_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Door Contact value.
            pub fn get_door_contact(&self) -> Option<Type07PropDoorContact> {
                let raw_value = self.get_door_contact_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Lock Contact value.
            pub fn get_lock_contact_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Lock Contact value.
            pub fn get_lock_contact(&self) -> Option<Type07PropLockContact> {
                let raw_value = self.get_lock_contact_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type07<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type07")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("door_contact", &self.get_door_contact())
                    .field("lock_contact", &self.get_lock_contact())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type07PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type07PropDoorContact {
            DoorClosed = false,
            DoorOpen = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type07PropLockContact {
            DoorLocked = false,
            DoorUnlocked = true,
            _Other(bool),
        }
        /// Dual-door-contact with States Open/Closed and Locked/Unlocked, Supply voltage monitor and Vibration detection (A5-14-08)
        #[derive(Clone, Copy)]
        pub struct Type08<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type08<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type08PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 5.0))
            }

            /// Get the raw Door Contact value.
            pub fn get_door_contact_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Door Contact value.
            pub fn get_door_contact(&self) -> Option<Type08PropDoorContact> {
                let raw_value = self.get_door_contact_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Lock Contact value.
            pub fn get_lock_contact_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Lock Contact value.
            pub fn get_lock_contact(&self) -> Option<Type08PropLockContact> {
                let raw_value = self.get_lock_contact_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Vibration value.
            pub fn get_vibration_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Vibration value.
            pub fn get_vibration(&self) -> Option<Type08PropVibration> {
                let raw_value = self.get_vibration_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type08<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type08")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("door_contact", &self.get_door_contact())
                    .field("lock_contact", &self.get_lock_contact())
                    .field("vibration", &self.get_vibration())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08PropDoorContact {
            DoorClosed = false,
            DoorOpen = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08PropLockContact {
            DoorLocked = false,
            DoorUnlocked = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08PropVibration {
            NoVibrationDetected = false,
            VibrationDetected = true,
            _Other(bool),
        }
        /// Window/Door-Sensor with States Open/Closed/Tilt, Supply voltage monitor (A5-14-09)
        #[derive(Clone, Copy)]
        pub struct Type09<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type09<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type09PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 5.0))
            }

            /// Get the raw Contact value.
            pub fn get_contact_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(29, 2)
            }
            /// Get the Contact value.
            pub fn get_contact(&self) -> Option<Type09PropContact> {
                let raw_value = self.get_contact_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type09<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type09")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("contact", &self.get_contact())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type09PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type09PropContact {
            Closed = 0,
            Tilt = 1,
            Reserved = 2,
            Open = 3,
            _Other(u8),
        }
        /// Window/Door-Sensor with States Open/Closed/Tilt, Supply voltage monitor and Vibration detection (A5-14-0A)
        #[derive(Clone, Copy)]
        pub struct Type0A<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type0A<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type0APropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 250.0, 0.0, 5.0))
            }

            /// Get the raw Contact value.
            pub fn get_contact_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(29, 2)
            }
            /// Get the Contact value.
            pub fn get_contact(&self) -> Option<Type0APropContact> {
                let raw_value = self.get_contact_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Vibration value.
            pub fn get_vibration_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Vibration value.
            pub fn get_vibration(&self) -> Option<Type0APropVibration> {
                let raw_value = self.get_vibration_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type0A<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type0A")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("contact", &self.get_contact())
                    .field("vibration", &self.get_vibration())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0APropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type0APropContact {
            Closed = 0,
            Tilt = 1,
            Reserved = 2,
            Open = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type0APropVibration {
            NoVibrationDetected = false,
            VibrationDetected = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func14<'b> {
        Type01(func14::Type01<'b>),
        Type02(func14::Type02<'b>),
        Type03(func14::Type03<'b>),
        Type04(func14::Type04<'b>),
        Type05(func14::Type05<'b>),
        Type06(func14::Type06<'b>),
        Type07(func14::Type07<'b>),
        Type08(func14::Type08<'b>),
        Type09(func14::Type09<'b>),
        Type0A(func14::Type0A<'b>),
    }
    impl<'b> Func14<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func14::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02(
                func14::Type02::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type03_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type03(
                func14::Type03::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type04_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type04(
                func14::Type04::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type05_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type05(
                func14::Type05::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type06_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type06(
                func14::Type06::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type07_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type07(
                func14::Type07::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type08_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type08(
                func14::Type08::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type09_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type09(
                func14::Type09::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type0A_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type0A(
                func14::Type0A::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x03 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type03_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x04 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type04_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x05 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type05_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x06 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type06_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x07 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type07_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x08 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type08_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x09 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type09_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x0A => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type0A_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// HVAC Components (A5-20)
    pub mod func20 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Battery Powered Actuator (A5-20-01), case 0
        #[derive(Clone, Copy)]
        pub struct Type01Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type01Case0PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Current Value value.
            pub fn get_current_value_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Current Value value in units of %.
            pub fn get_current_value(&self) -> Option<f64> {
                let raw_value = self.get_current_value_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 100.0, 0.0, 100.0))
            }

            /// Get the raw Service On value.
            pub fn get_service_on_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(8, 1)
            }
            /// Get the Service On value.
            pub fn get_service_on(&self) -> Option<Type01Case0PropServiceOn> {
                let raw_value = self.get_service_on_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy input enabled value.
            pub fn get_energy_input_enabled_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(9, 1)
            }
            /// Get the Energy input enabled value.
            pub fn get_energy_input_enabled(&self) -> Option<Type01Case0PropEnergyInputEnabled> {
                let raw_value = self.get_energy_input_enabled_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy Storage value.
            pub fn get_energy_storage_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(10, 1)
            }
            /// Get the Energy Storage value.
            pub fn get_energy_storage(&self) -> Option<Type01Case0PropEnergyStorage> {
                let raw_value = self.get_energy_storage_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Battery capacity value.
            pub fn get_battery_capacity_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(11, 1)
            }
            /// Get the Battery capacity value.
            pub fn get_battery_capacity(&self) -> Option<Type01Case0PropBatteryCapacity> {
                let raw_value = self.get_battery_capacity_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Contact, cover open value.
            pub fn get_contact_cover_open_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(12, 1)
            }
            /// Get the Contact, cover open value.
            pub fn get_contact_cover_open(&self) -> Option<Type01Case0PropContactCoverOpen> {
                let raw_value = self.get_contact_cover_open_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Failure temperature sensor, out off range value.
            pub fn get_failure_temperature_sensor_out_off_range_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(13, 1)
            }
            /// Get the Failure temperature sensor, out off range value.
            pub fn get_failure_temperature_sensor_out_off_range(&self) -> Option<Type01Case0PropFailureTemperatureSensorOutOffRange> {
                let raw_value = self.get_failure_temperature_sensor_out_off_range_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Detection, window open value.
            pub fn get_detection_window_open_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(14, 1)
            }
            /// Get the Detection, window open value.
            pub fn get_detection_window_open(&self) -> Option<Type01Case0PropDetectionWindowOpen> {
                let raw_value = self.get_detection_window_open_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Actuator obstructed value.
            pub fn get_actuator_obstructed_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(15, 1)
            }
            /// Get the Actuator obstructed value.
            pub fn get_actuator_obstructed(&self) -> Option<Type01Case0PropActuatorObstructed> {
                let raw_value = self.get_actuator_obstructed_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 40.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type01Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01Case0")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("current_value", &self.get_current_value())
                    .field("service_on", &self.get_service_on())
                    .field("energy_input_enabled", &self.get_energy_input_enabled())
                    .field("energy_storage", &self.get_energy_storage())
                    .field("battery_capacity", &self.get_battery_capacity())
                    .field("contact_cover_open", &self.get_contact_cover_open())
                    .field("failure_temperature_sensor_out_off_range", &self.get_failure_temperature_sensor_out_off_range())
                    .field("detection_window_open", &self.get_detection_window_open())
                    .field("actuator_obstructed", &self.get_actuator_obstructed())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case0PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case0PropServiceOn {
            On = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case0PropEnergyInputEnabled {
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case0PropEnergyStorage {
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case0PropBatteryCapacity {
            True = false,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case0PropContactCoverOpen {
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case0PropFailureTemperatureSensorOutOffRange {
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case0PropDetectionWindowOpen {
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case0PropActuatorObstructed {
            True = true,
            _Other(bool),
        }
        /// Battery Powered Actuator (A5-20-01), case 1
        #[derive(Clone, Copy)]
        pub struct Type01Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type01Case1PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Valve position or Temperature Setpoint value.
            pub fn get_valve_position_or_temperature_setpoint_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }

            /// Get the raw Temperature  from RCU value.
            pub fn get_temperature_from_rcu_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Temperature  from RCU value in units of °C.
            pub fn get_temperature_from_rcu(&self) -> Option<f64> {
                let raw_value = self.get_temperature_from_rcu_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 0.0, 40.0))
            }

            /// Get the raw Run init sequence value.
            pub fn get_run_init_sequence_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(16, 1)
            }
            /// Get the Run init sequence value.
            pub fn get_run_init_sequence(&self) -> Option<Type01Case1PropRunInitSequence> {
                let raw_value = self.get_run_init_sequence_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Lift set value.
            pub fn get_lift_set_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(17, 1)
            }
            /// Get the Lift set value.
            pub fn get_lift_set(&self) -> Option<Type01Case1PropLiftSet> {
                let raw_value = self.get_lift_set_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Valve open / maintenance value.
            pub fn get_valve_open_maintenance_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(18, 1)
            }
            /// Get the Valve open / maintenance value.
            pub fn get_valve_open_maintenance(&self) -> Option<Type01Case1PropValveOpenMaintenance> {
                let raw_value = self.get_valve_open_maintenance_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Valve closed value.
            pub fn get_valve_closed_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(19, 1)
            }
            /// Get the Valve closed value.
            pub fn get_valve_closed(&self) -> Option<Type01Case1PropValveClosed> {
                let raw_value = self.get_valve_closed_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Summer bit, Reduction of energy consumption value.
            pub fn get_summer_bit_reduction_of_energy_consumption_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(20, 1)
            }
            /// Get the Summer bit, Reduction of energy consumption value.
            pub fn get_summer_bit_reduction_of_energy_consumption(&self) -> Option<Type01Case1PropSummerBitReductionOfEnergyConsumption> {
                let raw_value = self.get_summer_bit_reduction_of_energy_consumption_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Set Point Selection value.
            pub fn get_set_point_selection_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(21, 1)
            }
            /// Get the Set Point Selection value.
            pub fn get_set_point_selection(&self) -> Option<Type01Case1PropSetPointSelection> {
                let raw_value = self.get_set_point_selection_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Set point inverse value.
            pub fn get_set_point_inverse_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(22, 1)
            }
            /// Get the Set point inverse value.
            pub fn get_set_point_inverse(&self) -> Option<Type01Case1PropSetPointInverse> {
                let raw_value = self.get_set_point_inverse_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Select function value.
            pub fn get_select_function_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(23, 1)
            }
            /// Get the Select function value.
            pub fn get_select_function(&self) -> Option<Type01Case1PropSelectFunction> {
                let raw_value = self.get_select_function_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01Case1")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("valve_position_or_temperature_setpoint", &self.get_valve_position_or_temperature_setpoint_raw())
                    .field("temperature_from_rcu", &self.get_temperature_from_rcu())
                    .field("run_init_sequence", &self.get_run_init_sequence())
                    .field("lift_set", &self.get_lift_set())
                    .field("valve_open_maintenance", &self.get_valve_open_maintenance())
                    .field("valve_closed", &self.get_valve_closed())
                    .field("summer_bit_reduction_of_energy_consumption", &self.get_summer_bit_reduction_of_energy_consumption())
                    .field("set_point_selection", &self.get_set_point_selection())
                    .field("set_point_inverse", &self.get_set_point_inverse())
                    .field("select_function", &self.get_select_function())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropRunInitSequence {
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropLiftSet {
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropValveOpenMaintenance {
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropValveClosed {
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropSummerBitReductionOfEnergyConsumption {
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropSetPointSelection {
            ValvePosition0Minus100UnitRespondToController = false,
            TemperatureSetPoint040DegreesCUnitRespondToRoomSensorAndUseInternalPiLoop = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropSetPointInverse {
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropSelectFunction {
            Rcu = false,
            ServiceOn = true,
            _Other(bool),
        }
        /// Basic Actuator (A5-20-02), case 0
        #[derive(Clone, Copy)]
        pub struct Type02Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type02Case0PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Actual Value value.
            pub fn get_actual_value_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Actual Value value in units of %.
            pub fn get_actual_value(&self) -> Option<f64> {
                let raw_value = self.get_actual_value_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 100.0, 0.0, 100.0))
            }

            /// Get the raw Set point inverse value.
            pub fn get_set_point_inverse_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(22, 1)
            }
            /// Get the Set point inverse value.
            pub fn get_set_point_inverse(&self) -> Option<Type02Case0PropSetPointInverse> {
                let raw_value = self.get_set_point_inverse_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02Case0")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("actual_value", &self.get_actual_value())
                    .field("set_point_inverse", &self.get_set_point_inverse())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02Case0PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02Case0PropSetPointInverse {
            True = true,
            _Other(bool),
        }
        /// Basic Actuator (A5-20-02), case 1
        #[derive(Clone, Copy)]
        pub struct Type02Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type02Case1PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Valve Set point value.
            pub fn get_valve_set_point_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Valve Set point value in units of %.
            pub fn get_valve_set_point(&self) -> Option<f64> {
                let raw_value = self.get_valve_set_point_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 100.0, 0.0, 100.0))
            }

            /// Get the raw Set point inverse value.
            pub fn get_set_point_inverse_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(22, 1)
            }
            /// Get the Set point inverse value.
            pub fn get_set_point_inverse(&self) -> Option<Type02Case1PropSetPointInverse> {
                let raw_value = self.get_set_point_inverse_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02Case1")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("valve_set_point", &self.get_valve_set_point())
                    .field("set_point_inverse", &self.get_set_point_inverse())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02Case1PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02Case1PropSetPointInverse {
            True = true,
            _Other(bool),
        }
        /// Line powered Actuator (A5-20-03), case 0
        #[derive(Clone, Copy)]
        pub struct Type03Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type03Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type03Case0PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Actual valve value.
            pub fn get_actual_valve_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Actual valve value in units of %.
            pub fn get_actual_valve(&self) -> Option<f64> {
                let raw_value = self.get_actual_valve_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 100.0, 0.0, 100.0))
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 40.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type03Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type03Case0")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("actual_valve", &self.get_actual_valve())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03Case0PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Line powered Actuator (A5-20-03), case 1
        #[derive(Clone, Copy)]
        pub struct Type03Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type03Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Set Point Inverse value.
            pub fn get_set_point_inverse_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(22, 1)
            }
            /// Get the Set Point Inverse value.
            pub fn get_set_point_inverse(&self) -> Option<Type03Case1PropSetPointInverse> {
                let raw_value = self.get_set_point_inverse_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Set Point Selection value.
            pub fn get_set_point_selection_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(21, 1)
            }
            /// Get the Set Point Selection value.
            pub fn get_set_point_selection(&self) -> Option<Type03Case1PropSetPointSelection> {
                let raw_value = self.get_set_point_selection_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type03Case1PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Actuator or Temperature Setpoint value.
            pub fn get_actuator_or_temperature_setpoint_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }

            /// Get the raw Temperature  from RCU value.
            pub fn get_temperature_from_rcu_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Temperature  from RCU value in units of °C.
            pub fn get_temperature_from_rcu(&self) -> Option<f64> {
                let raw_value = self.get_temperature_from_rcu_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 0.0, 40.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type03Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type03Case1")
                    .field("set_point_inverse", &self.get_set_point_inverse())
                    .field("set_point_selection", &self.get_set_point_selection())
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("actuator_or_temperature_setpoint", &self.get_actuator_or_temperature_setpoint_raw())
                    .field("temperature_from_rcu", &self.get_temperature_from_rcu())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03Case1PropSetPointInverse {
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03Case1PropSetPointSelection {
            ActuatorSetpoint0Minus100UnitRespondToController = false,
            TemperatureSetpoint040DegreesCUnitRespondToRoomSensorAndUseInternalPiLoop = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03Case1PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Heating Radiator Valve Actuating Drive with Feed and Room Temperature Measurement, Local Set Point Control and Display (A5-20-04), case 0
        #[derive(Clone, Copy)]
        pub struct Type04Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type04Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type04Case0PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Current Position value.
            pub fn get_current_position_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Current Position value in units of %.
            pub fn get_current_position(&self) -> Option<f64> {
                let raw_value = self.get_current_position_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 100.0, 0.0, 100.0))
            }

            /// Get the raw Feed Temperature OR Temperature Set Point value.
            pub fn get_feed_temperature_or_temperature_set_point_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }

            /// Get the raw Room Temperature OR Failure Code value.
            pub fn get_room_temperature_or_failure_code_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Room Temperature OR Failure Code value.
            pub fn get_room_temperature_or_failure_code(&self) -> Option<Type04Case0PropRoomTemperatureOrFailureCode> {
                let raw_value = self.get_room_temperature_or_failure_code_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Measurement Status value.
            pub fn get_measurement_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(24, 1)
            }
            /// Get the Measurement Status value.
            pub fn get_measurement_status(&self) -> Option<Type04Case0PropMeasurementStatus> {
                let raw_value = self.get_measurement_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Status Request value.
            pub fn get_status_request_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(25, 1)
            }
            /// Get the Status Request value.
            pub fn get_status_request(&self) -> Option<Type04Case0PropStatusRequest> {
                let raw_value = self.get_status_request_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Button Lock Status value.
            pub fn get_button_lock_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Button Lock Status value.
            pub fn get_button_lock_status(&self) -> Option<Type04Case0PropButtonLockStatus> {
                let raw_value = self.get_button_lock_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature Selection value.
            pub fn get_temperature_selection_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Temperature Selection value.
            pub fn get_temperature_selection(&self) -> Option<Type04Case0PropTemperatureSelection> {
                let raw_value = self.get_temperature_selection_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Failure value.
            pub fn get_failure_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Failure value.
            pub fn get_failure(&self) -> Option<Type04Case0PropFailure> {
                let raw_value = self.get_failure_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type04Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type04Case0")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("current_position", &self.get_current_position())
                    .field("feed_temperature_or_temperature_set_point", &self.get_feed_temperature_or_temperature_set_point_raw())
                    .field("room_temperature_or_failure_code", &self.get_room_temperature_or_failure_code())
                    .field("measurement_status", &self.get_measurement_status())
                    .field("status_request", &self.get_status_request())
                    .field("button_lock_status", &self.get_button_lock_status())
                    .field("temperature_selection", &self.get_temperature_selection())
                    .field("failure", &self.get_failure())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04Case0PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type04Case0PropRoomTemperatureOrFailureCode {
            MeasurementError = 17,
            BatteryEmpty = 18,
            Reserved = 19,
            FrostProtection = 20,
            BlockedValve = 33,
            EndPointDetectionError = 36,
            NoValve = 40,
            NotTaughtIn = 49,
            NoResponseFromController = 53,
            TeachInError = 54,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04Case0PropMeasurementStatus {
            Active = false,
            Inactive = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04Case0PropStatusRequest {
            NoChange = false,
            StatusRequested = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04Case0PropButtonLockStatus {
            Unlocked = false,
            Locked = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04Case0PropTemperatureSelection {
            FeedTemperature = false,
            TemperatureSetPoint = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04Case0PropFailure {
            NoFailureTmpIsTransmitted = false,
            FailureFcIsTransmitted = true,
            _Other(bool),
        }
        /// Heating Radiator Valve Actuating Drive with Feed and Room Temperature Measurement, Local Set Point Control and Display (A5-20-04), case 1
        #[derive(Clone, Copy)]
        pub struct Type04Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type04Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Valve Position value.
            pub fn get_valve_position_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Valve Position value in units of %.
            pub fn get_valve_position(&self) -> Option<f64> {
                let raw_value = self.get_valve_position_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 100.0, 0.0, 100.0))
            }

            /// Get the raw Temperature Set Point value.
            pub fn get_temperature_set_point_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Temperature Set Point value in units of °C.
            pub fn get_temperature_set_point(&self) -> Option<f64> {
                let raw_value = self.get_temperature_set_point_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 10.0, 30.0))
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type04Case1PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Measurement Control value.
            pub fn get_measurement_control_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(17, 1)
            }
            /// Get the Measurement Control value.
            pub fn get_measurement_control(&self) -> Option<Type04Case1PropMeasurementControl> {
                let raw_value = self.get_measurement_control_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Wake-up Cycle value.
            pub fn get_wake_up_cycle_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(18, 6)
            }
            /// Get the Wake-up Cycle value.
            pub fn get_wake_up_cycle(&self) -> Option<Type04Case1PropWakeUpCycle> {
                let raw_value = self.get_wake_up_cycle_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Display Orientation value.
            pub fn get_display_orientation_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(26, 2)
            }
            /// Get the Display Orientation value.
            pub fn get_display_orientation(&self) -> Option<Type04Case1PropDisplayOrientation> {
                let raw_value = self.get_display_orientation_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Button Lock Control value.
            pub fn get_button_lock_control_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Button Lock Control value.
            pub fn get_button_lock_control(&self) -> Option<Type04Case1PropButtonLockControl> {
                let raw_value = self.get_button_lock_control_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Service Command value.
            pub fn get_service_command_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(30, 2)
            }
            /// Get the Service Command value.
            pub fn get_service_command(&self) -> Option<Type04Case1PropServiceCommand> {
                let raw_value = self.get_service_command_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type04Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type04Case1")
                    .field("valve_position", &self.get_valve_position())
                    .field("temperature_set_point", &self.get_temperature_set_point())
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("measurement_control", &self.get_measurement_control())
                    .field("wake_up_cycle", &self.get_wake_up_cycle())
                    .field("display_orientation", &self.get_display_orientation())
                    .field("button_lock_control", &self.get_button_lock_control())
                    .field("service_command", &self.get_service_command())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04Case1PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04Case1PropMeasurementControl {
            Enable = false,
            Disable = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type04Case1PropWakeUpCycle {
            _10Sec = 0,
            _60Sec = 1,
            _90Sec = 2,
            _120Sec = 3,
            _150Sec = 4,
            _180Sec = 5,
            _210Sec = 6,
            _240Sec = 7,
            _270Sec = 8,
            _300Sec5Min = 9,
            _330Sec = 10,
            _360Sec = 11,
            _390Sec = 12,
            _420Sec = 13,
            _450Sec = 14,
            _480Sec = 15,
            _510Sec = 16,
            _540Sec = 17,
            _570Sec = 18,
            _600Sec10Min = 19,
            _630Sec = 20,
            _660Sec = 21,
            _690Sec = 22,
            _720Sec = 23,
            _750Sec = 24,
            _780Sec = 25,
            _810Sec = 26,
            _840Sec = 27,
            _870Sec = 28,
            _900Sec15Min = 29,
            _930Sec = 30,
            _960Sec = 31,
            _990Sec = 32,
            _1020Sec = 33,
            _1050Sec = 34,
            _1080Sec = 35,
            _1110Sec = 36,
            _1140Sec = 37,
            _1170Sec = 38,
            _1200Sec20Min = 39,
            _1230Sec = 40,
            _1260Sec = 41,
            _1290Sec = 42,
            _1320Sec = 43,
            _1350Sec = 44,
            _1380Sec = 45,
            _1410Sec = 46,
            _1440Sec = 47,
            _1470Sec = 48,
            _1500Sec25Min = 49,
            _3Hrs = 50,
            _6Hrs = 51,
            _9Hrs = 52,
            _12Hrs = 53,
            _15Hrs = 54,
            _18Hrs = 55,
            _21Hrs = 56,
            _24Hrs = 57,
            _27Hrs = 58,
            _30Hrs = 59,
            _33Hrs = 60,
            _36Hrs = 61,
            _39Hrs = 62,
            _42HrsMax = 63,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type04Case1PropDisplayOrientation {
            _0Degrees = 0,
            _90Degrees = 1,
            _180Degrees = 2,
            _270Degrees = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04Case1PropButtonLockControl {
            Unlocked = false,
            Locked = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type04Case1PropServiceCommand {
            NoChange = 0,
            OpenValve = 1,
            RunInitialisation = 2,
            CloseValve = 3,
            _Other(u8),
        }
        /// Ventilation Unit (A5-20-05), case 0
        #[derive(Clone, Copy)]
        pub struct Type05Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type05Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type05Case0PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Actual speed setting value.
            pub fn get_actual_speed_setting_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 3)
            }
            /// Get the Actual speed setting value.
            pub fn get_actual_speed_setting(&self) -> Option<Type05Case0PropActualSpeedSetting> {
                let raw_value = self.get_actual_speed_setting_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Actual speed timer setting value.
            pub fn get_actual_speed_timer_setting_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(3, 3)
            }
            /// Get the Actual speed timer setting value.
            pub fn get_actual_speed_timer_setting(&self) -> Option<Type05Case0PropActualSpeedTimerSetting> {
                let raw_value = self.get_actual_speed_timer_setting_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Node low battery value.
            pub fn get_node_low_battery_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(11, 1)
            }
            /// Get the Node low battery value.
            pub fn get_node_low_battery(&self) -> Option<Type05Case0PropNodeLowBattery> {
                let raw_value = self.get_node_low_battery_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Node comm. error value.
            pub fn get_node_comm_error_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(12, 1)
            }
            /// Get the Node comm. error value.
            pub fn get_node_comm_error(&self) -> Option<Type05Case0PropNodeCommError> {
                let raw_value = self.get_node_comm_error_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Sensor error value.
            pub fn get_sensor_error_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(13, 1)
            }
            /// Get the Sensor error value.
            pub fn get_sensor_error(&self) -> Option<Type05Case0PropSensorError> {
                let raw_value = self.get_sensor_error_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Fan speed error value.
            pub fn get_fan_speed_error_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(14, 1)
            }
            /// Get the Fan speed error value.
            pub fn get_fan_speed_error(&self) -> Option<Type05Case0PropFanSpeedError> {
                let raw_value = self.get_fan_speed_error_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Error value.
            pub fn get_error_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(15, 1)
            }
            /// Get the Error value.
            pub fn get_error(&self) -> Option<Type05Case0PropError> {
                let raw_value = self.get_error_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Filter condition value.
            pub fn get_filter_condition_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Filter condition value.
            pub fn get_filter_condition(&self) -> Option<Type05Case0PropFilterCondition> {
                let raw_value = self.get_filter_condition_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Auto speed supported value.
            pub fn get_auto_speed_supported_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(24, 1)
            }
            /// Get the Auto speed supported value.
            pub fn get_auto_speed_supported(&self) -> Option<Type05Case0PropAutoSpeedSupported> {
                let raw_value = self.get_auto_speed_supported_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw By-pass active value.
            pub fn get_by_pass_active_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(25, 1)
            }
            /// Get the By-pass active value.
            pub fn get_by_pass_active(&self) -> Option<Type05Case0PropByPassActive> {
                let raw_value = self.get_by_pass_active_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Frost protection value.
            pub fn get_frost_protection_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(26, 1)
            }
            /// Get the Frost protection value.
            pub fn get_frost_protection(&self) -> Option<Type05Case0PropFrostProtection> {
                let raw_value = self.get_frost_protection_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type05Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type05Case0")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("actual_speed_setting", &self.get_actual_speed_setting())
                    .field("actual_speed_timer_setting", &self.get_actual_speed_timer_setting())
                    .field("node_low_battery", &self.get_node_low_battery())
                    .field("node_comm_error", &self.get_node_comm_error())
                    .field("sensor_error", &self.get_sensor_error())
                    .field("fan_speed_error", &self.get_fan_speed_error())
                    .field("error", &self.get_error())
                    .field("filter_condition", &self.get_filter_condition())
                    .field("auto_speed_supported", &self.get_auto_speed_supported())
                    .field("by_pass_active", &self.get_by_pass_active())
                    .field("frost_protection", &self.get_frost_protection())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case0PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type05Case0PropActualSpeedSetting {
            MinimumSpeedAway = 0,
            Speed1Low = 1,
            Speed2Mid = 2,
            Speed3High = 3,
            MaxSpeed = 4,
            Auto = 5,
            NotUsed = 6,
            NotUsed1 = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type05Case0PropActualSpeedTimerSetting {
            NoTimerSetOrExpired = 0,
            _110MinLeft = 1,
            _1120MinLeft = 2,
            _2130MinLeft = 3,
            _3140MinLeft = 4,
            _4150MinLeft = 5,
            _5160MinLeft = 6,
            MoreThan = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case0PropNodeLowBattery {
            NoError = false,
            Error = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case0PropNodeCommError {
            NoError = false,
            Error = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case0PropSensorError {
            NoError = false,
            Error = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case0PropFanSpeedError {
            NoError = false,
            Error = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case0PropError {
            NoError = false,
            Error = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type05Case0PropFilterCondition {
            FilterNeedsReplacement = 0,
            NoFilterPresent = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case0PropAutoSpeedSupported {
            AutoSpeedNotSupported = false,
            AutoSpeedSupported = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case0PropByPassActive {
            SummerBypassNotActive = false,
            SummerBypassActive = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case0PropFrostProtection {
            FrostProtectionNotActive = false,
            FrostProtectionActive = true,
            _Other(bool),
        }
        /// Ventilation Unit (A5-20-05), case 1
        #[derive(Clone, Copy)]
        pub struct Type05Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type05Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type05Case1PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw New speed setting value.
            pub fn get_new_speed_setting_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 3)
            }
            /// Get the New speed setting value.
            pub fn get_new_speed_setting(&self) -> Option<Type05Case1PropNewSpeedSetting> {
                let raw_value = self.get_new_speed_setting_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw New speed timer setting value.
            pub fn get_new_speed_timer_setting_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(3, 5)
            }
            /// Get the New speed timer setting value.
            pub fn get_new_speed_timer_setting(&self) -> Option<Type05Case1PropNewSpeedTimerSetting> {
                let raw_value = self.get_new_speed_timer_setting_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Reset error value.
            pub fn get_reset_error_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(24, 1)
            }
            /// Get the Reset error value.
            pub fn get_reset_error(&self) -> Option<Type05Case1PropResetError> {
                let raw_value = self.get_reset_error_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Reset filter timer value.
            pub fn get_reset_filter_timer_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(25, 1)
            }
            /// Get the Reset filter timer value.
            pub fn get_reset_filter_timer(&self) -> Option<Type05Case1PropResetFilterTimer> {
                let raw_value = self.get_reset_filter_timer_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type05Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type05Case1")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("new_speed_setting", &self.get_new_speed_setting())
                    .field("new_speed_timer_setting", &self.get_new_speed_timer_setting())
                    .field("reset_error", &self.get_reset_error())
                    .field("reset_filter_timer", &self.get_reset_filter_timer())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case1PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type05Case1PropNewSpeedSetting {
            MinimumSpeedAway = 0,
            Speed1Low = 1,
            Speed2Mid = 2,
            Speed3High = 3,
            MaxSpeed = 4,
            Auto = 5,
            NotUsed = 6,
            NoChange = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type05Case1PropNewSpeedTimerSetting {
            NoTimerNoChange = 0,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case1PropResetError {
            DonTResetError = false,
            ResetError = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05Case1PropResetFilterTimer {
            DonTResetFilterTimer = false,
            ResetFilterTimer = true,
            _Other(bool),
        }
        /// Harvesting-powered actuator with local temperature offset control (BI-DIR) (A5-20-06), case 0
        #[derive(Clone, Copy)]
        pub struct Type06Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type06Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Current Value value.
            pub fn get_current_value_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Current Value value in units of %.
            pub fn get_current_value(&self) -> Option<f64> {
                let raw_value = self.get_current_value_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 100.0, 0.0, 100.0))
            }

            /// Get the raw Local Offset Mode value.
            pub fn get_local_offset_mode_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(8, 1)
            }
            /// Get the Local Offset Mode value.
            pub fn get_local_offset_mode(&self) -> Option<Type06Case0PropLocalOffsetMode> {
                let raw_value = self.get_local_offset_mode_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Local Offset value.
            pub fn get_local_offset_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(9, 7)
            }
            /// Get the Local Offset value.
            pub fn get_local_offset(&self) -> Option<Type06Case0PropLocalOffset> {
                let raw_value = self.get_local_offset_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Temperature value.
            pub fn get_temperature(&self) -> Option<Type06Case0PropTemperature> {
                let raw_value = self.get_temperature_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Tempertature Selection value.
            pub fn get_tempertature_selection_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(24, 1)
            }
            /// Get the Tempertature Selection value.
            pub fn get_tempertature_selection(&self) -> Option<Type06Case0PropTempertatureSelection> {
                let raw_value = self.get_tempertature_selection_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy Input Enabled value.
            pub fn get_energy_input_enabled_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(25, 1)
            }
            /// Get the Energy Input Enabled value.
            pub fn get_energy_input_enabled(&self) -> Option<Type06Case0PropEnergyInputEnabled> {
                let raw_value = self.get_energy_input_enabled_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy Storage value.
            pub fn get_energy_storage_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(26, 1)
            }
            /// Get the Energy Storage value.
            pub fn get_energy_storage(&self) -> Option<Type06Case0PropEnergyStorage> {
                let raw_value = self.get_energy_storage_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Window open detection value.
            pub fn get_window_open_detection_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(27, 1)
            }
            /// Get the Window open detection value.
            pub fn get_window_open_detection(&self) -> Option<Type06Case0PropWindowOpenDetection> {
                let raw_value = self.get_window_open_detection_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type06Case0PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Radio Com Error value.
            pub fn get_radio_com_error_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Radio Com Error value.
            pub fn get_radio_com_error(&self) -> Option<Type06Case0PropRadioComError> {
                let raw_value = self.get_radio_com_error_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Radio Signal strength value.
            pub fn get_radio_signal_strength_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Radio Signal strength value.
            pub fn get_radio_signal_strength(&self) -> Option<Type06Case0PropRadioSignalStrength> {
                let raw_value = self.get_radio_signal_strength_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Actuator obstructed value.
            pub fn get_actuator_obstructed_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Actuator obstructed value.
            pub fn get_actuator_obstructed(&self) -> Option<Type06Case0PropActuatorObstructed> {
                let raw_value = self.get_actuator_obstructed_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type06Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type06Case0")
                    .field("current_value", &self.get_current_value())
                    .field("local_offset_mode", &self.get_local_offset_mode())
                    .field("local_offset", &self.get_local_offset())
                    .field("temperature", &self.get_temperature())
                    .field("tempertature_selection", &self.get_tempertature_selection())
                    .field("energy_input_enabled", &self.get_energy_input_enabled())
                    .field("energy_storage", &self.get_energy_storage())
                    .field("window_open_detection", &self.get_window_open_detection())
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("radio_com_error", &self.get_radio_com_error())
                    .field("radio_signal_strength", &self.get_radio_signal_strength())
                    .field("actuator_obstructed", &self.get_actuator_obstructed())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06Case0PropLocalOffsetMode {
            LoIsRelativeOffsettemperature = false,
            LoIsAbsoluteAbsoluttemperature = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type06Case0PropLocalOffset {
            LocalOffset0c = 0,
            LocalOffset1c = 1,
            LocalOffset2c = 2,
            LocalOffset3c = 3,
            LocalOffset4c = 4,
            LocalOffset5c = 5,
            LocalOffsetMinus5c = 123,
            LocalOffsetMinus4c = 124,
            LocalOffsetMinus3c = 125,
            LocalOffsetMinus2cc = 126,
            LocalOffsetMinus1c = 127,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type06Case0PropTemperature {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06Case0PropTempertatureSelection {
            AmbientSensorTemperature = false,
            FeedSensorTemperature = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06Case0PropEnergyInputEnabled {
            NotHarvesting = false,
            HarvestingActive = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06Case0PropEnergyStorage {
            LowAlmostDischarged = false,
            SufficentlyCharged = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06Case0PropWindowOpenDetection {
            NoWindowOpenDetected = false,
            WindowOpenDetected = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06Case0PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06Case0PropRadioComError {
            RadioCommunicationIsStable = false,
            _6OrMoreConsecutiveCommunicationErrosHaveOccured = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06Case0PropRadioSignalStrength {
            RadioSignalIsStrong = false,
            RadioSignalIsWeakUnderMinus77dbm = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06Case0PropActuatorObstructed {
            ActuatorWorkingCorrectly = false,
            ActuatorBlocked = true,
            _Other(bool),
        }
        /// Harvesting-powered actuator with local temperature offset control (BI-DIR) (A5-20-06), case 1
        #[derive(Clone, Copy)]
        pub struct Type06Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type06Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Setpoint value.
            pub fn get_setpoint_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Setpoint value.
            pub fn get_setpoint(&self) -> Option<Type06Case1PropSetpoint> {
                let raw_value = self.get_setpoint_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Room temperature from contrul unit value.
            pub fn get_room_temperature_from_contrul_unit_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Room temperature from contrul unit value in units of %.
            pub fn get_room_temperature_from_contrul_unit(&self) -> Option<f64> {
                let raw_value = self.get_room_temperature_from_contrul_unit_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 160.0, 0.0, 40.0))
            }

            /// Get the raw Reference run value.
            pub fn get_reference_run_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(16, 1)
            }
            /// Get the Reference run value.
            pub fn get_reference_run(&self) -> Option<Type06Case1PropReferenceRun> {
                let raw_value = self.get_reference_run_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw RF Communication intervall value.
            pub fn get_rf_communication_intervall_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(17, 4)
            }
            /// Get the RF Communication intervall value.
            pub fn get_rf_communication_intervall(&self) -> Option<Type06Case1PropRfCommunicationIntervall> {
                let raw_value = self.get_rf_communication_intervall_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Initiate summer mode Bit value.
            pub fn get_initiate_summer_mode_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(20, 1)
            }
            /// Get the Initiate summer mode Bit value.
            pub fn get_initiate_summer_mode_bit(&self) -> Option<Type06Case1PropInitiateSummerModeBit> {
                let raw_value = self.get_initiate_summer_mode_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Set point selection value.
            pub fn get_set_point_selection_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(21, 1)
            }
            /// Get the Set point selection value.
            pub fn get_set_point_selection(&self) -> Option<Type06Case1PropSetPointSelection> {
                let raw_value = self.get_set_point_selection_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature Selection value.
            pub fn get_temperature_selection_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(22, 1)
            }
            /// Get the Temperature Selection value.
            pub fn get_temperature_selection(&self) -> Option<Type06Case1PropTemperatureSelection> {
                let raw_value = self.get_temperature_selection_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Standbye value.
            pub fn get_standbye_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(23, 1)
            }
            /// Get the Standbye value.
            pub fn get_standbye(&self) -> Option<Type06Case1PropStandbye> {
                let raw_value = self.get_standbye_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type06Case1PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type06Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type06Case1")
                    .field("setpoint", &self.get_setpoint())
                    .field("room_temperature_from_contrul_unit", &self.get_room_temperature_from_contrul_unit())
                    .field("reference_run", &self.get_reference_run())
                    .field("rf_communication_intervall", &self.get_rf_communication_intervall())
                    .field("initiate_summer_mode_bit", &self.get_initiate_summer_mode_bit())
                    .field("set_point_selection", &self.get_set_point_selection())
                    .field("temperature_selection", &self.get_temperature_selection())
                    .field("standbye", &self.get_standbye())
                    .field("lrn_bit", &self.get_lrn_bit())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type06Case1PropSetpoint {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06Case1PropReferenceRun {
            NormalOperation = false,
            ReferenceRun = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type06Case1PropRfCommunicationIntervall {
            Auto = 0,
            _2Minutes = 1,
            _5Minutes = 2,
            _10Minutes = 3,
            _20Minutes = 4,
            _30Minutes = 5,
            _60Minutes = 6,
            _120Minutes = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06Case1PropInitiateSummerModeBit {
            NormalOperation = false,
            SummerModeWith8hoursRadioDutyCycle = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06Case1PropSetPointSelection {
            ValvePositionMode = false,
            TemperatureSetpoint = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06Case1PropTemperatureSelection {
            RequestAmbientTemperature = false,
            RequestFeedtemperature = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06Case1PropStandbye {
            NormalOperation = false,
            EnterStandbye = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06Case1PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        /// Generic HVAC Interface (A5-20-10), case 0
        #[derive(Clone, Copy)]
        pub struct Type10Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type10Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type10Case0PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Mode value.
            pub fn get_mode_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Mode value.
            pub fn get_mode(&self) -> Option<Type10Case0PropMode> {
                let raw_value = self.get_mode_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Vane position value.
            pub fn get_vane_position_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 4)
            }
            /// Get the Vane position value.
            pub fn get_vane_position(&self) -> Option<Type10Case0PropVanePosition> {
                let raw_value = self.get_vane_position_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Fan Speed value.
            pub fn get_fan_speed_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(12, 4)
            }
            /// Get the Fan Speed value.
            pub fn get_fan_speed(&self) -> Option<Type10Case0PropFanSpeed> {
                let raw_value = self.get_fan_speed_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Control variable value.
            pub fn get_control_variable_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }

            /// Get the raw Room occupancy value.
            pub fn get_room_occupancy_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(29, 2)
            }
            /// Get the Room occupancy value.
            pub fn get_room_occupancy(&self) -> Option<Type10Case0PropRoomOccupancy> {
                let raw_value = self.get_room_occupancy_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw On/Off value.
            pub fn get_on_off_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the On/Off value.
            pub fn get_on_off(&self) -> Option<Type10Case0PropOnOff> {
                let raw_value = self.get_on_off_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type10Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type10Case0")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("mode", &self.get_mode())
                    .field("vane_position", &self.get_vane_position())
                    .field("fan_speed", &self.get_fan_speed())
                    .field("control_variable", &self.get_control_variable_raw())
                    .field("room_occupancy", &self.get_room_occupancy())
                    .field("on_off", &self.get_on_off())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type10Case0PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type10Case0PropMode {
            Auto = 0,
            Heat = 1,
            MorningWarmup = 2,
            Cool = 3,
            NightPurge = 4,
            Precool = 5,
            Off = 6,
            Test = 7,
            EmergencyHeat = 8,
            FanOnly = 9,
            FreeCool = 10,
            Ice = 11,
            MaxHeat = 12,
            EconomicHeatCool = 13,
            DehumidificationDry = 14,
            Calibration = 15,
            EmergencyCool = 16,
            EmergencySteam = 17,
            MaxCool = 18,
            HvcLoad = 19,
            NoLoad = 20,
            AutoHeat = 31,
            AutoCool = 32,
            NA = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type10Case0PropVanePosition {
            Auto = 0,
            Horizontal = 1,
            Pos2 = 2,
            Pos3 = 3,
            Pos4 = 4,
            Vertical = 5,
            Swing = 6,
            VerticalSwing = 11,
            HorizontalSwing = 12,
            HorizontalAndVerticalSwing = 13,
            StopSwing = 14,
            NA = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type10Case0PropFanSpeed {
            Auto = 0,
            NA = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type10Case0PropRoomOccupancy {
            Occupied = 0,
            StandbyWaitingToPerformAction = 1,
            UnoccupiedActionPerformed = 2,
            OffNoOccupancyAndNoAction = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type10Case0PropOnOff {
            OffTheUnitIsNotRunning = false,
            On = true,
            _Other(bool),
        }
        /// Generic HVAC Interface (A5-20-10), case 1
        #[derive(Clone, Copy)]
        pub struct Type10Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type10Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type10Case1PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Mode value.
            pub fn get_mode_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Mode value.
            pub fn get_mode(&self) -> Option<Type10Case1PropMode> {
                let raw_value = self.get_mode_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Vane position value.
            pub fn get_vane_position_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 4)
            }
            /// Get the Vane position value.
            pub fn get_vane_position(&self) -> Option<Type10Case1PropVanePosition> {
                let raw_value = self.get_vane_position_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Fan Speed value.
            pub fn get_fan_speed_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(12, 4)
            }
            /// Get the Fan Speed value.
            pub fn get_fan_speed(&self) -> Option<Type10Case1PropFanSpeed> {
                let raw_value = self.get_fan_speed_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Control variable value.
            pub fn get_control_variable_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }

            /// Get the raw Room occupancy value.
            pub fn get_room_occupancy_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(29, 2)
            }
            /// Get the Room occupancy value.
            pub fn get_room_occupancy(&self) -> Option<Type10Case1PropRoomOccupancy> {
                let raw_value = self.get_room_occupancy_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw On/Off value.
            pub fn get_on_off_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the On/Off value.
            pub fn get_on_off(&self) -> Option<Type10Case1PropOnOff> {
                let raw_value = self.get_on_off_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type10Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type10Case1")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("mode", &self.get_mode())
                    .field("vane_position", &self.get_vane_position())
                    .field("fan_speed", &self.get_fan_speed())
                    .field("control_variable", &self.get_control_variable_raw())
                    .field("room_occupancy", &self.get_room_occupancy())
                    .field("on_off", &self.get_on_off())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type10Case1PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type10Case1PropMode {
            Auto = 0,
            Heat = 1,
            MorningWarmup = 2,
            Cool = 3,
            NightPurge = 4,
            Precool = 5,
            Off = 6,
            Test = 7,
            EmergencyHeat = 8,
            FanOnly = 9,
            FreeCool = 10,
            Ice = 11,
            MaxHeat = 12,
            EconomicHeatCool = 13,
            DehumidificationDry = 14,
            Calibration = 15,
            EmergencyCool = 16,
            EmergencySteam = 17,
            MaxCool = 18,
            HvcLoad = 19,
            NoLoad = 20,
            AutoHeat = 31,
            AutoCool = 32,
            NA = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type10Case1PropVanePosition {
            Auto = 0,
            Horizontal = 1,
            Pos2 = 2,
            Pos3 = 3,
            Pos4 = 4,
            Vertical = 5,
            Swing = 6,
            VerticalSwing = 11,
            HorizontalSwing = 12,
            HorizontalAndVerticalSwing = 13,
            StopSwing = 14,
            NA = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type10Case1PropFanSpeed {
            Auto = 0,
            NA = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type10Case1PropRoomOccupancy {
            Occupied = 0,
            StandbyWaitingToPerformAction = 1,
            UnoccupiedActionPerformed = 2,
            OffNoOccupancyAndNoAction = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type10Case1PropOnOff {
            Off = false,
            On = true,
            _Other(bool),
        }
        /// Generic HVAC Interface – Error Control (A5-20-11), case 0
        #[derive(Clone, Copy)]
        pub struct Type11Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type11Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Disable remote controller value.
            pub fn get_disable_remote_controller_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Disable remote controller value.
            pub fn get_disable_remote_controller(&self) -> Option<Type11Case0PropDisableRemoteController> {
                let raw_value = self.get_disable_remote_controller_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type11Case0PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw External disablement value.
            pub fn get_external_disablement_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(23, 1)
            }
            /// Get the External disablement value.
            pub fn get_external_disablement(&self) -> Option<Type11Case0PropExternalDisablement> {
                let raw_value = self.get_external_disablement_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Window contact value.
            pub fn get_window_contact_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Window contact value.
            pub fn get_window_contact(&self) -> Option<Type11Case0PropWindowContact> {
                let raw_value = self.get_window_contact_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type11Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type11Case0")
                    .field("disable_remote_controller", &self.get_disable_remote_controller())
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("external_disablement", &self.get_external_disablement())
                    .field("window_contact", &self.get_window_contact())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type11Case0PropDisableRemoteController {
            EnableRemoteController = false,
            DisableRemoteController = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type11Case0PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type11Case0PropExternalDisablement {
            NotDisabled = false,
            Disabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type11Case0PropWindowContact {
            WindowsOpened = false,
            WindowsClosed = true,
            _Other(bool),
        }
        /// Generic HVAC Interface – Error Control (A5-20-11), case 1
        #[derive(Clone, Copy)]
        pub struct Type11Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type11Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Remote controller Disablement value.
            pub fn get_remote_controller_disablement_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Remote controller Disablement value.
            pub fn get_remote_controller_disablement(&self) -> Option<Type11Case1PropRemoteControllerDisablement> {
                let raw_value = self.get_remote_controller_disablement_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type11Case1PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Error Code value.
            pub fn get_error_code_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(0, 16)
            }
            /// Get the Error Code value in units of N/A.
            pub fn get_error_code(&self) -> Option<f64> {
                let raw_value = self.get_error_code_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 65535.0, 0.0, 65535.0))
            }

            /// Get the raw Reserved value.
            pub fn get_reserved_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 4)
            }
            /// Get the Reserved value.
            pub fn get_reserved(&self) -> Option<Type11Case1PropReserved> {
                let raw_value = self.get_reserved_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Other disablement value.
            pub fn get_other_disablement_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(20, 1)
            }
            /// Get the Other disablement value.
            pub fn get_other_disablement(&self) -> Option<Type11Case1PropOtherDisablement> {
                let raw_value = self.get_other_disablement_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Window contact disablement value.
            pub fn get_window_contact_disablement_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(21, 1)
            }
            /// Get the Window contact disablement value.
            pub fn get_window_contact_disablement(&self) -> Option<Type11Case1PropWindowContactDisablement> {
                let raw_value = self.get_window_contact_disablement_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Key card disablement value.
            pub fn get_key_card_disablement_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(22, 1)
            }
            /// Get the Key card disablement value.
            pub fn get_key_card_disablement(&self) -> Option<Type11Case1PropKeyCardDisablement> {
                let raw_value = self.get_key_card_disablement_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw External disablement value.
            pub fn get_external_disablement_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(23, 1)
            }
            /// Get the External disablement value.
            pub fn get_external_disablement(&self) -> Option<Type11Case1PropExternalDisablement> {
                let raw_value = self.get_external_disablement_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Window contact value.
            pub fn get_window_contact_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Window contact value.
            pub fn get_window_contact(&self) -> Option<Type11Case1PropWindowContact> {
                let raw_value = self.get_window_contact_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Alarm State value.
            pub fn get_alarm_state_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Alarm State value.
            pub fn get_alarm_state(&self) -> Option<Type11Case1PropAlarmState> {
                let raw_value = self.get_alarm_state_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type11Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type11Case1")
                    .field("remote_controller_disablement", &self.get_remote_controller_disablement())
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("error_code", &self.get_error_code())
                    .field("reserved", &self.get_reserved())
                    .field("other_disablement", &self.get_other_disablement())
                    .field("window_contact_disablement", &self.get_window_contact_disablement())
                    .field("key_card_disablement", &self.get_key_card_disablement())
                    .field("external_disablement", &self.get_external_disablement())
                    .field("window_contact", &self.get_window_contact())
                    .field("alarm_state", &self.get_alarm_state())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type11Case1PropRemoteControllerDisablement {
            RemoteControllerEnabled = false,
            RemoteControllerDisabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type11Case1PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type11Case1PropReserved {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type11Case1PropOtherDisablement {
            NotDisabled = false,
            Disabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type11Case1PropWindowContactDisablement {
            NotDisabled = false,
            Disabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type11Case1PropKeyCardDisablement {
            NotDisabled = false,
            Disabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type11Case1PropExternalDisablement {
            NotDisabled = false,
            Disabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type11Case1PropWindowContact {
            WindowsOpened = false,
            WindowsClosed = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type11Case1PropAlarmState {
            Ok = false,
            Error = true,
            _Other(bool),
        }
        /// Temperature Controller Input (A5-20-12)
        #[derive(Clone, Copy)]
        pub struct Type12<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type12<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type12PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Control Variable override value.
            pub fn get_control_variable_override_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Control Variable override value in units of %.
            pub fn get_control_variable_override(&self) -> Option<f64> {
                let raw_value = self.get_control_variable_override_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 100.0))
            }

            /// Get the raw FanStage override value.
            pub fn get_fanstage_override_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the FanStage override value.
            pub fn get_fanstage_override(&self) -> Option<Type12PropFanstageOverride> {
                let raw_value = self.get_fanstage_override_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Setpoint shift value.
            pub fn get_setpoint_shift_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Setpoint shift value in units of °K.
            pub fn get_setpoint_shift(&self) -> Option<f64> {
                let raw_value = self.get_setpoint_shift_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, -10.0, 10.0))
            }

            /// Get the raw Fan override value.
            pub fn get_fan_override_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(24, 1)
            }
            /// Get the Fan override value.
            pub fn get_fan_override(&self) -> Option<Type12PropFanOverride> {
                let raw_value = self.get_fan_override_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Controller mode value.
            pub fn get_controller_mode_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(25, 2)
            }
            /// Get the Controller mode value.
            pub fn get_controller_mode(&self) -> Option<Type12PropControllerMode> {
                let raw_value = self.get_controller_mode_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Controller state value.
            pub fn get_controller_state_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(27, 1)
            }
            /// Get the Controller state value.
            pub fn get_controller_state(&self) -> Option<Type12PropControllerState> {
                let raw_value = self.get_controller_state_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy hold-off / Dew point value.
            pub fn get_energy_hold_off_dew_point_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Energy hold-off / Dew point value.
            pub fn get_energy_hold_off_dew_point(&self) -> Option<Type12PropEnergyHoldOffDewPoint> {
                let raw_value = self.get_energy_hold_off_dew_point_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Room occupancy value.
            pub fn get_room_occupancy_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(30, 2)
            }
            /// Get the Room occupancy value.
            pub fn get_room_occupancy(&self) -> Option<Type12PropRoomOccupancy> {
                let raw_value = self.get_room_occupancy_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type12<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type12")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("control_variable_override", &self.get_control_variable_override())
                    .field("fanstage_override", &self.get_fanstage_override())
                    .field("setpoint_shift", &self.get_setpoint_shift())
                    .field("fan_override", &self.get_fan_override())
                    .field("controller_mode", &self.get_controller_mode())
                    .field("controller_state", &self.get_controller_state())
                    .field("energy_hold_off_dew_point", &self.get_energy_hold_off_dew_point())
                    .field("room_occupancy", &self.get_room_occupancy())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type12PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type12PropFanstageOverride {
            Stage0 = 0,
            Stage1 = 1,
            Stage2 = 2,
            Stage3 = 3,
            Auto = 31,
            NotAvailable = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type12PropFanOverride {
            Automatic = false,
            OverrideFanDb2 = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type12PropControllerMode {
            AutoMode = 0,
            Heating = 1,
            Cooling = 2,
            Off = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type12PropControllerState {
            Automatic = false,
            OverrideControlVariableDb3 = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type12PropEnergyHoldOffDewPoint {
            Normal = false,
            EnergyHoldOffDewPoint = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type12PropRoomOccupancy {
            Occupied = 0,
            Unoccupied = 1,
            Standby = 2,
            Frost = 3,
            _Other(u8),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func20<'b> {
        Type01Case0(func20::Type01Case0<'b>),
        Type01Case1(func20::Type01Case1<'b>),
        Type02Case0(func20::Type02Case0<'b>),
        Type02Case1(func20::Type02Case1<'b>),
        Type03Case0(func20::Type03Case0<'b>),
        Type03Case1(func20::Type03Case1<'b>),
        Type04Case0(func20::Type04Case0<'b>),
        Type04Case1(func20::Type04Case1<'b>),
        Type05Case0(func20::Type05Case0<'b>),
        Type05Case1(func20::Type05Case1<'b>),
        Type06Case0(func20::Type06Case0<'b>),
        Type06Case1(func20::Type06Case1<'b>),
        Type10Case0(func20::Type10Case0<'b>),
        Type10Case1(func20::Type10Case1<'b>),
        Type11Case0(func20::Type11Case0<'b>),
        Type11Case1(func20::Type11Case1<'b>),
        Type12(func20::Type12<'b>),
    }
    impl<'b> Func20<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 2> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01Case0(
                func20::Type01Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type01Case1(
                func20::Type01Case1::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 2> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02Case0(
                func20::Type02Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type02Case1(
                func20::Type02Case1::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type03_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 2> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type03Case0(
                func20::Type03Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type03Case1(
                func20::Type03Case1::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type04_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 2> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type04Case0(
                func20::Type04Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type04Case1(
                func20::Type04Case1::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type05_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 2> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type05Case0(
                func20::Type05Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type05Case1(
                func20::Type05Case1::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type06_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 2> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type06Case0(
                func20::Type06Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type06Case1(
                func20::Type06Case1::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type10_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 2> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type10Case0(
                func20::Type10Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type10Case1(
                func20::Type10Case1::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type11_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 2> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type11Case0(
                func20::Type11Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type11Case1(
                func20::Type11Case1::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type12_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type12(
                func20::Type12::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 2>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x03 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type03_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x04 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type04_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x05 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type05_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x06 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type06_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x10 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type10_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x11 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type11_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x12 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type12_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Digital Input (A5-30)
    pub mod func30 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Single Input Contact, Battery Monitor (A5-30-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type01PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Supply voltage value.
            pub fn get_supply_voltage(&self) -> Option<Type01PropSupplyVoltage> {
                let raw_value = self.get_supply_voltage_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Input State value.
            pub fn get_input_state_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Input State value.
            pub fn get_input_state(&self) -> Option<Type01PropInputState> {
                let raw_value = self.get_input_state_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("input_state", &self.get_input_state())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropSupplyVoltage {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropInputState {
            _Other(u8),
        }
        /// Single Input Contact (A5-30-02)
        #[derive(Clone, Copy)]
        pub struct Type02<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type02PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Input State value.
            pub fn get_input_state_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Input State value.
            pub fn get_input_state(&self) -> Option<Type02PropInputState> {
                let raw_value = self.get_input_state_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("input_state", &self.get_input_state())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropInputState {
            ContactClosed = false,
            ContactOpen = true,
            _Other(bool),
        }
        /// 4 Digital Inputs, Wake and Temperature (A5-30-03)
        #[derive(Clone, Copy)]
        pub struct Type03<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type03<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type03PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Temperature value in units of °C.
            pub fn get_temperature(&self) -> Option<f64> {
                let raw_value = self.get_temperature_raw()? as f64;
                Some(range_scale(raw_value, 255.0, 0.0, 0.0, 40.0))
            }

            /// Get the raw Status of Wake value.
            pub fn get_status_of_wake_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(19, 1)
            }
            /// Get the Status of Wake value.
            pub fn get_status_of_wake(&self) -> Option<Type03PropStatusOfWake> {
                let raw_value = self.get_status_of_wake_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Digital Input 3 value.
            pub fn get_digital_input_3_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(20, 1)
            }
            /// Get the Digital Input 3 value.
            pub fn get_digital_input_3(&self) -> Option<Type03PropDigitalInput3> {
                let raw_value = self.get_digital_input_3_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Digital Input 2 value.
            pub fn get_digital_input_2_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(21, 1)
            }
            /// Get the Digital Input 2 value.
            pub fn get_digital_input_2(&self) -> Option<Type03PropDigitalInput2> {
                let raw_value = self.get_digital_input_2_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Digital Input 1 value.
            pub fn get_digital_input_1_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(22, 1)
            }
            /// Get the Digital Input 1 value.
            pub fn get_digital_input_1(&self) -> Option<Type03PropDigitalInput1> {
                let raw_value = self.get_digital_input_1_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Digital Input 0 value.
            pub fn get_digital_input_0_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(23, 1)
            }
            /// Get the Digital Input 0 value.
            pub fn get_digital_input_0(&self) -> Option<Type03PropDigitalInput0> {
                let raw_value = self.get_digital_input_0_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type03<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type03")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temperature", &self.get_temperature())
                    .field("status_of_wake", &self.get_status_of_wake())
                    .field("digital_input_3", &self.get_digital_input_3())
                    .field("digital_input_2", &self.get_digital_input_2())
                    .field("digital_input_1", &self.get_digital_input_1())
                    .field("digital_input_0", &self.get_digital_input_0())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropStatusOfWake {
            Low = false,
            High = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropDigitalInput3 {
            Low = false,
            High = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropDigitalInput2 {
            Low = false,
            High = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropDigitalInput1 {
            Low = false,
            High = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type03PropDigitalInput0 {
            Low = false,
            High = true,
            _Other(bool),
        }
        /// 3 Digital Inputs, 1 Digital Input 8 Bits (A5-30-04)
        #[derive(Clone, Copy)]
        pub struct Type04<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type04<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type04PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Digital value-input value.
            pub fn get_digital_value_input_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Digital value-input value in units of N/A.
            pub fn get_digital_value_input(&self) -> Option<f64> {
                let raw_value = self.get_digital_value_input_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 255.0))
            }

            /// Get the raw Digital Input 2 value.
            pub fn get_digital_input_2_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Digital Input 2 value.
            pub fn get_digital_input_2(&self) -> Option<Type04PropDigitalInput2> {
                let raw_value = self.get_digital_input_2_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Digital Input 1 value.
            pub fn get_digital_input_1_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Digital Input 1 value.
            pub fn get_digital_input_1(&self) -> Option<Type04PropDigitalInput1> {
                let raw_value = self.get_digital_input_1_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Digital Input 0 value.
            pub fn get_digital_input_0_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Digital Input 0 value.
            pub fn get_digital_input_0(&self) -> Option<Type04PropDigitalInput0> {
                let raw_value = self.get_digital_input_0_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type04<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type04")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("digital_value_input", &self.get_digital_value_input())
                    .field("digital_input_2", &self.get_digital_input_2())
                    .field("digital_input_1", &self.get_digital_input_1())
                    .field("digital_input_0", &self.get_digital_input_0())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropDigitalInput2 {
            Low = false,
            High = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropDigitalInput1 {
            Low = false,
            High = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type04PropDigitalInput0 {
            Low = false,
            High = true,
            _Other(bool),
        }
        /// Single Input Contact, Retransmission, Battery Monitor (A5-30-05)
        #[derive(Clone, Copy)]
        pub struct Type05<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type05<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type05PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply voltage value.
            pub fn get_supply_voltage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Supply voltage value in units of V.
            pub fn get_supply_voltage(&self) -> Option<f64> {
                let raw_value = self.get_supply_voltage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 3.3))
            }

            /// Get the raw Signal type value.
            pub fn get_signal_type_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(16, 1)
            }
            /// Get the Signal type value.
            pub fn get_signal_type(&self) -> Option<Type05PropSignalType> {
                let raw_value = self.get_signal_type_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Index of Signals value.
            pub fn get_index_of_signals_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(17, 7)
            }
            /// Get the Index of Signals value.
            pub fn get_index_of_signals(&self) -> Option<Type05PropIndexOfSignals> {
                let raw_value = self.get_index_of_signals_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type05<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type05")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("supply_voltage", &self.get_supply_voltage())
                    .field("signal_type", &self.get_signal_type())
                    .field("index_of_signals", &self.get_index_of_signals())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type05PropSignalType {
            NormalSignal = false,
            HeartBeatSignal = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type05PropIndexOfSignals {
            _Other(u8),
        }
        /// Single Alternate Input Contact, Retransmission, Heartbeat (A5-30-06)
        #[derive(Clone, Copy)]
        pub struct Type06<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type06<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw contact value.
            pub fn get_contact_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(16, 1)
            }
            /// Get the contact value.
            pub fn get_contact(&self) -> Option<Type06PropContact> {
                let raw_value = self.get_contact_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Index value.
            pub fn get_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(17, 3)
            }
            /// Get the Index value.
            pub fn get_index(&self) -> Option<f64> {
                let raw_value = self.get_index_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 7.0, 0.0, 7.0))
            }

            /// Get the raw Type value.
            pub fn get_type_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(20, 1)
            }
            /// Get the Type value.
            pub fn get_type(&self) -> Option<Type06PropType> {
                let raw_value = self.get_type_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Retransmission value.
            pub fn get_retransmission_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(21, 3)
            }
            /// Get the Retransmission value.
            pub fn get_retransmission(&self) -> Option<f64> {
                let raw_value = self.get_retransmission_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 7.0, 0.0, 7.0))
            }

            /// Get the raw LrnBit value.
            pub fn get_lrnbit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LrnBit value.
            pub fn get_lrnbit(&self) -> Option<Type06PropLrnbit> {
                let raw_value = self.get_lrnbit_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type06<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type06")
                    .field("contact", &self.get_contact())
                    .field("index", &self.get_index())
                    .field("type", &self.get_type())
                    .field("retransmission", &self.get_retransmission())
                    .field("lrnbit", &self.get_lrnbit())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06PropContact {
            Open = false,
            Close = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06PropType {
            TriggerEvent = false,
            Heartbeat = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type06PropLrnbit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func30<'b> {
        Type01(func30::Type01<'b>),
        Type02(func30::Type02<'b>),
        Type03(func30::Type03<'b>),
        Type04(func30::Type04<'b>),
        Type05(func30::Type05<'b>),
        Type06(func30::Type06<'b>),
    }
    impl<'b> Func30<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func30::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02(
                func30::Type02::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type03_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type03(
                func30::Type03::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type04_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type04(
                func30::Type04::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type05_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type05(
                func30::Type05::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type06_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type06(
                func30::Type06::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x03 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type03_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x04 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type04_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x05 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type05_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x06 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type06_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Energy Management (A5-37)
    pub mod func37 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Demand Response (A5-37-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw DR Level value.
            pub fn get_dr_level_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the DR Level value in units of N/A.
            pub fn get_dr_level(&self) -> Option<f64> {
                let raw_value = self.get_dr_level_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 15.0, 0.0, 15.0))
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type01PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temporary default value.
            pub fn get_temporary_default_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Temporary default value in units of N/A.
            pub fn get_temporary_default(&self) -> Option<f64> {
                let raw_value = self.get_temporary_default_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 255.0))
            }

            /// Get the raw Absolute/relative power usage value.
            pub fn get_absolute_relative_power_usage_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(8, 1)
            }
            /// Get the Absolute/relative power usage value.
            pub fn get_absolute_relative_power_usage(&self) -> Option<Type01PropAbsoluteRelativePowerUsage> {
                let raw_value = self.get_absolute_relative_power_usage_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Power Usage value.
            pub fn get_power_usage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(9, 7)
            }
            /// Get the Power Usage value in units of N/A.
            pub fn get_power_usage(&self) -> Option<f64> {
                let raw_value = self.get_power_usage_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 100.0, 0.0, 100.0))
            }

            /// Get the raw Timeout Setting value.
            pub fn get_timeout_setting_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Timeout Setting value in units of min.
            pub fn get_timeout_setting(&self) -> Option<f64> {
                let raw_value = self.get_timeout_setting_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 255.0, 15.0, 3825.0))
            }

            /// Get the raw Random start delay value.
            pub fn get_random_start_delay_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Random start delay value.
            pub fn get_random_start_delay(&self) -> Option<Type01PropRandomStartDelay> {
                let raw_value = self.get_random_start_delay_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Randomized end delay value.
            pub fn get_randomized_end_delay_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Randomized end delay value.
            pub fn get_randomized_end_delay(&self) -> Option<Type01PropRandomizedEndDelay> {
                let raw_value = self.get_randomized_end_delay_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Max/Min Power Usage for Default DR State value.
            pub fn get_max_min_power_usage_for_default_dr_state_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Max/Min Power Usage for Default DR State value.
            pub fn get_max_min_power_usage_for_default_dr_state(&self) -> Option<Type01PropMaxMinPowerUsageForDefaultDrState> {
                let raw_value = self.get_max_min_power_usage_for_default_dr_state_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("dr_level", &self.get_dr_level())
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("temporary_default", &self.get_temporary_default())
                    .field("absolute_relative_power_usage", &self.get_absolute_relative_power_usage())
                    .field("power_usage", &self.get_power_usage())
                    .field("timeout_setting", &self.get_timeout_setting())
                    .field("random_start_delay", &self.get_random_start_delay())
                    .field("randomized_end_delay", &self.get_randomized_end_delay())
                    .field("max_min_power_usage_for_default_dr_state", &self.get_max_min_power_usage_for_default_dr_state())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropAbsoluteRelativePowerUsage {
            AbsolutePowerUsageInterpretDb_2Bit_6Db_2Bit_0AsAPercentageOfTheMaximumPowerUse = false,
            RelativePowerUsageInterpretDb_2Bit_6Db_2Bit_0AsAPercentageOfTheCurrentPowerUse = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropRandomStartDelay {
            False = false,
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropRandomizedEndDelay {
            False = false,
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropMaxMinPowerUsageForDefaultDrState {
            MinimumPowerUsage = false,
            MaximumPowerUsage = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func37<'b> {
        Type01(func37::Type01<'b>),
    }
    impl<'b> Func37<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func37::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Central Command (A5-38)
    pub mod func38 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Gateway (A5-38-08), case 0
        #[derive(Clone, Copy)]
        pub struct Type08Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type08Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type08Case0PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command value.
            pub fn get_command_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Command value.
            pub fn get_command(&self) -> Option<Type08Case0PropCommand> {
                let raw_value = self.get_command_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Time value.
            pub fn get_time_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(8, 16)
            }
            /// Get the Time value in units of s.
            pub fn get_time(&self) -> Option<f64> {
                let raw_value = self.get_time_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 65535.0, 0.1, 6553.5))
            }

            /// Get the raw Lock/Unlock value.
            pub fn get_lock_unlock_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Lock/Unlock value.
            pub fn get_lock_unlock(&self) -> Option<Type08Case0PropLockUnlock> {
                let raw_value = self.get_lock_unlock_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Delay or duration value.
            pub fn get_delay_or_duration_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Delay or duration value.
            pub fn get_delay_or_duration(&self) -> Option<Type08Case0PropDelayOrDuration> {
                let raw_value = self.get_delay_or_duration_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Switching Command value.
            pub fn get_switching_command_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Switching Command value.
            pub fn get_switching_command(&self) -> Option<Type08Case0PropSwitchingCommand> {
                let raw_value = self.get_switching_command_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type08Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type08Case0")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("command", &self.get_command())
                    .field("time", &self.get_time())
                    .field("lock_unlock", &self.get_lock_unlock())
                    .field("delay_or_duration", &self.get_delay_or_duration())
                    .field("switching_command", &self.get_switching_command())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case0PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type08Case0PropCommand {
            Value1 = 1,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case0PropLockUnlock {
            Unlock = false,
            Lock = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case0PropDelayOrDuration {
            Duration = false,
            Delay = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case0PropSwitchingCommand {
            Off = false,
            On = true,
            _Other(bool),
        }
        /// Gateway (A5-38-08), case 1
        #[derive(Clone, Copy)]
        pub struct Type08Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type08Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type08Case1PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command value.
            pub fn get_command_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Command value.
            pub fn get_command(&self) -> Option<Type08Case1PropCommand> {
                let raw_value = self.get_command_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Dimming value value.
            pub fn get_dimming_value_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Dimming value value in units of %.
            pub fn get_dimming_value(&self) -> Option<f64> {
                let raw_value = self.get_dimming_value_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 100.0))
            }

            /// Get the raw Ramping time value.
            pub fn get_ramping_time_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Ramping time value in units of s.
            pub fn get_ramping_time(&self) -> Option<f64> {
                let raw_value = self.get_ramping_time_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 255.0))
            }

            /// Get the raw Dimming Range value.
            pub fn get_dimming_range_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Dimming Range value.
            pub fn get_dimming_range(&self) -> Option<Type08Case1PropDimmingRange> {
                let raw_value = self.get_dimming_range_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Store final value value.
            pub fn get_store_final_value_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Store final value value.
            pub fn get_store_final_value(&self) -> Option<Type08Case1PropStoreFinalValue> {
                let raw_value = self.get_store_final_value_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Switching Command value.
            pub fn get_switching_command_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Switching Command value.
            pub fn get_switching_command(&self) -> Option<Type08Case1PropSwitchingCommand> {
                let raw_value = self.get_switching_command_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type08Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type08Case1")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("command", &self.get_command())
                    .field("dimming_value", &self.get_dimming_value())
                    .field("ramping_time", &self.get_ramping_time())
                    .field("dimming_range", &self.get_dimming_range())
                    .field("store_final_value", &self.get_store_final_value())
                    .field("switching_command", &self.get_switching_command())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case1PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type08Case1PropCommand {
            Value2 = 2,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case1PropDimmingRange {
            AbsoluteValue = false,
            RelativeValue = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case1PropStoreFinalValue {
            No = false,
            Yes = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case1PropSwitchingCommand {
            Off = false,
            On = true,
            _Other(bool),
        }
        /// Gateway (A5-38-08), case 2
        #[derive(Clone, Copy)]
        pub struct Type08Case2<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type08Case2<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type08Case2PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command value.
            pub fn get_command_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Command value.
            pub fn get_command(&self) -> Option<Type08Case2PropCommand> {
                let raw_value = self.get_command_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Setpoint value.
            pub fn get_setpoint_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Setpoint value in units of K.
            pub fn get_setpoint(&self) -> Option<f64> {
                let raw_value = self.get_setpoint_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, -12.7, 12.8))
            }
        }
        impl<'b> ::core::fmt::Debug for Type08Case2<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type08Case2")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("command", &self.get_command())
                    .field("setpoint", &self.get_setpoint())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case2PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type08Case2PropCommand {
            Value3 = 3,
            _Other(u8),
        }
        /// Gateway (A5-38-08), case 3
        #[derive(Clone, Copy)]
        pub struct Type08Case3<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type08Case3<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type08Case3PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command value.
            pub fn get_command_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Command value.
            pub fn get_command(&self) -> Option<Type08Case3PropCommand> {
                let raw_value = self.get_command_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Basic Setpoint value.
            pub fn get_basic_setpoint_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Basic Setpoint value in units of °C.
            pub fn get_basic_setpoint(&self) -> Option<f64> {
                let raw_value = self.get_basic_setpoint_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 51.2))
            }
        }
        impl<'b> ::core::fmt::Debug for Type08Case3<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type08Case3")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("command", &self.get_command())
                    .field("basic_setpoint", &self.get_basic_setpoint())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case3PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type08Case3PropCommand {
            Value4 = 4,
            _Other(u8),
        }
        /// Gateway (A5-38-08), case 4
        #[derive(Clone, Copy)]
        pub struct Type08Case4<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type08Case4<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type08Case4PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command value.
            pub fn get_command_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Command value.
            pub fn get_command(&self) -> Option<Type08Case4PropCommand> {
                let raw_value = self.get_command_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Control variable override value.
            pub fn get_control_variable_override_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Control variable override value in units of %.
            pub fn get_control_variable_override(&self) -> Option<f64> {
                let raw_value = self.get_control_variable_override_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 100.0))
            }

            /// Get the raw Controller mode value.
            pub fn get_controller_mode_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(25, 2)
            }
            /// Get the Controller mode value.
            pub fn get_controller_mode(&self) -> Option<Type08Case4PropControllerMode> {
                let raw_value = self.get_controller_mode_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Controller state value.
            pub fn get_controller_state_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(27, 1)
            }
            /// Get the Controller state value.
            pub fn get_controller_state(&self) -> Option<Type08Case4PropControllerState> {
                let raw_value = self.get_controller_state_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy hold off value.
            pub fn get_energy_hold_off_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Energy hold off value.
            pub fn get_energy_hold_off(&self) -> Option<Type08Case4PropEnergyHoldOff> {
                let raw_value = self.get_energy_hold_off_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Room occupancy value.
            pub fn get_room_occupancy_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(30, 2)
            }
            /// Get the Room occupancy value.
            pub fn get_room_occupancy(&self) -> Option<Type08Case4PropRoomOccupancy> {
                let raw_value = self.get_room_occupancy_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type08Case4<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type08Case4")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("command", &self.get_command())
                    .field("control_variable_override", &self.get_control_variable_override())
                    .field("controller_mode", &self.get_controller_mode())
                    .field("controller_state", &self.get_controller_state())
                    .field("energy_hold_off", &self.get_energy_hold_off())
                    .field("room_occupancy", &self.get_room_occupancy())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case4PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type08Case4PropCommand {
            Value5 = 5,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type08Case4PropControllerMode {
            AutomaticModeSelection = 0,
            Heating = 1,
            Cooling = 2,
            Off = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case4PropControllerState {
            Automatic = false,
            Override = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case4PropEnergyHoldOff {
            Normal = false,
            EnergyHoldoffDewPoint = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type08Case4PropRoomOccupancy {
            Occupied = 0,
            Unoccupied = 1,
            Standby = 2,
            _Other(u8),
        }
        /// Gateway (A5-38-08), case 5
        #[derive(Clone, Copy)]
        pub struct Type08Case5<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type08Case5<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type08Case5PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command value.
            pub fn get_command_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Command value.
            pub fn get_command(&self) -> Option<Type08Case5PropCommand> {
                let raw_value = self.get_command_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw FanStage override value.
            pub fn get_fanstage_override_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the FanStage override value.
            pub fn get_fanstage_override(&self) -> Option<Type08Case5PropFanstageOverride> {
                let raw_value = self.get_fanstage_override_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type08Case5<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type08Case5")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("command", &self.get_command())
                    .field("fanstage_override", &self.get_fanstage_override())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case5PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type08Case5PropCommand {
            Value6 = 6,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type08Case5PropFanstageOverride {
            Stage0 = 0,
            Stage1 = 1,
            Stage2 = 2,
            Stage3 = 3,
            Auto = 255,
            _Other(u8),
        }
        /// Gateway (A5-38-08), case 6
        #[derive(Clone, Copy)]
        pub struct Type08Case6<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type08Case6<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type08Case6PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command value.
            pub fn get_command_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Command value.
            pub fn get_command(&self) -> Option<Type08Case6PropCommand> {
                let raw_value = self.get_command_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Parameter 1 value.
            pub fn get_parameter_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Parameter 1 value.
            pub fn get_parameter_1(&self) -> Option<Type08Case6PropParameter1> {
                let raw_value = self.get_parameter_1_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Parameter 2 value.
            pub fn get_parameter_2_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Parameter 2 value.
            pub fn get_parameter_2(&self) -> Option<Type08Case6PropParameter2> {
                let raw_value = self.get_parameter_2_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Function value.
            pub fn get_function_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Function value.
            pub fn get_function(&self) -> Option<Type08Case6PropFunction> {
                let raw_value = self.get_function_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Send status flag value.
            pub fn get_send_status_flag_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Send status flag value.
            pub fn get_send_status_flag(&self) -> Option<Type08Case6PropSendStatusFlag> {
                let raw_value = self.get_send_status_flag_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Pos. and Angle flag value.
            pub fn get_pos_and_angle_flag_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Pos. and Angle flag value.
            pub fn get_pos_and_angle_flag(&self) -> Option<Type08Case6PropPosAndAngleFlag> {
                let raw_value = self.get_pos_and_angle_flag_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Service Mode Flag value.
            pub fn get_service_mode_flag_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Service Mode Flag value.
            pub fn get_service_mode_flag(&self) -> Option<Type08Case6PropServiceModeFlag> {
                let raw_value = self.get_service_mode_flag_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type08Case6<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type08Case6")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("command", &self.get_command())
                    .field("parameter_1", &self.get_parameter_1())
                    .field("parameter_2", &self.get_parameter_2())
                    .field("function", &self.get_function())
                    .field("send_status_flag", &self.get_send_status_flag())
                    .field("pos_and_angle_flag", &self.get_pos_and_angle_flag())
                    .field("service_mode_flag", &self.get_service_mode_flag())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case6PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type08Case6PropCommand {
            ShuttersBlinds = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type08Case6PropParameter1 {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type08Case6PropParameter2 {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type08Case6PropFunction {
            DoNothingStatusRequest = 0,
            BlindStops = 1,
            BlindOpens = 2,
            BlindCloses = 3,
            BlindDrivesToPositionWithAngleValueSeeRemark2 = 4,
            BlindOpensForTimePositionValueAndAngleAngleValue = 5,
            BlindClosesForTimePositionValueAndAngleAngleValue = 6,
            SetRuntimeParametersSeeRemark3 = 7,
            SetAngleConfigurationSeeRemark3 = 8,
            SetMinMaxValuesSeeRemark4 = 9,
            SetSlatAngleForShutAndOpenPositionSeeRemark5 = 10,
            SetPositionLogicSeeRemark6 = 11,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case6PropSendStatusFlag {
            SendNewStatusOfDevice = false,
            SendNoStatusEGGlobalCentralCommands = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case6PropPosAndAngleFlag {
            NoAngleAndPositionValueAvailable = false,
            AngleAndPositionValueAvailable = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type08Case6PropServiceModeFlag {
            NormalOperation = false,
            ServiceModeTheModuleDisablesAllSendersExceptThisSenderWhichHasSetTheServiceModeForExampleForMaintenance = true,
            _Other(bool),
        }
        /// Extended Lighting-Control (A5-38-09)
        #[derive(Clone, Copy)]
        pub struct Type09<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type09<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type09PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Parameter 1 value.
            pub fn get_parameter_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Parameter 1 value.
            pub fn get_parameter_1(&self) -> Option<Type09PropParameter1> {
                let raw_value = self.get_parameter_1_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Parameter 2 value.
            pub fn get_parameter_2_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Parameter 2 value.
            pub fn get_parameter_2(&self) -> Option<Type09PropParameter2> {
                let raw_value = self.get_parameter_2_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Parameter 3 value.
            pub fn get_parameter_3_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Parameter 3 value.
            pub fn get_parameter_3(&self) -> Option<Type09PropParameter3> {
                let raw_value = self.get_parameter_3_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Function value.
            pub fn get_function_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Function value.
            pub fn get_function(&self) -> Option<Type09PropFunction> {
                let raw_value = self.get_function_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Send status flag value.
            pub fn get_send_status_flag_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(29, 1)
            }
            /// Get the Send status flag value.
            pub fn get_send_status_flag(&self) -> Option<Type09PropSendStatusFlag> {
                let raw_value = self.get_send_status_flag_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Store final value value.
            pub fn get_store_final_value_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(30, 1)
            }
            /// Get the Store final value value.
            pub fn get_store_final_value(&self) -> Option<Type09PropStoreFinalValue> {
                let raw_value = self.get_store_final_value_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Service Mode Flag value.
            pub fn get_service_mode_flag_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Service Mode Flag value.
            pub fn get_service_mode_flag(&self) -> Option<Type09PropServiceModeFlag> {
                let raw_value = self.get_service_mode_flag_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type09<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type09")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("parameter_1", &self.get_parameter_1())
                    .field("parameter_2", &self.get_parameter_2())
                    .field("parameter_3", &self.get_parameter_3())
                    .field("function", &self.get_function())
                    .field("send_status_flag", &self.get_send_status_flag())
                    .field("store_final_value", &self.get_store_final_value())
                    .field("service_mode_flag", &self.get_service_mode_flag())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type09PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type09PropParameter1 {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type09PropParameter2 {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type09PropParameter3 {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type09PropFunction {
            DoNothingStatusRequest = 0,
            SwitchedOff = 1,
            SwitchedOnMemoryValue = 2,
            DimmingUpWithRampingTime = 3,
            DimmingDownWithRampingTime = 4,
            DimmingStops = 5,
            SetDimmerValueAndRampingTime = 6,
            SetRgbValuesSeeRemark1 = 7,
            SceneFunctionSeeRemark2 = 8,
            SetMinimalAndMaximalDimmerValueSeeRemark3 = 9,
            SetTheOperatingHoursOfTheLampSeeRemark4 = 10,
            LockingLocalOperationsSeeRemark5 = 11,
            SetANewValueForTheEnergyMeteringOverwriteTheActualValueWithTheSelectedUnit = 12,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type09PropSendStatusFlag {
            SendNewStatusOfDevice = false,
            SendNoStatusEGGlobalCentralCommands = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type09PropStoreFinalValue {
            No = false,
            Yes = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type09PropServiceModeFlag {
            NormalOperation = false,
            ServiceModeTheModuleDisablesAllSendersExceptThisSenderWhichHasSetTheServiceModeForExampleForMaintenance = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func38<'b> {
        Type08Case0(func38::Type08Case0<'b>),
        Type08Case1(func38::Type08Case1<'b>),
        Type08Case2(func38::Type08Case2<'b>),
        Type08Case3(func38::Type08Case3<'b>),
        Type08Case4(func38::Type08Case4<'b>),
        Type08Case5(func38::Type08Case5<'b>),
        Type08Case6(func38::Type08Case6<'b>),
        Type09(func38::Type09<'b>),
    }
    impl<'b> Func38<'b> {
        pub fn type08_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 7> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type08Case0(
                func38::Type08Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type08Case1(
                func38::Type08Case1::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type08Case2(
                func38::Type08Case2::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type08Case3(
                func38::Type08Case3::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type08Case4(
                func38::Type08Case4::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type08Case5(
                func38::Type08Case5::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type08Case6(
                func38::Type08Case6::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type09_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type09(
                func38::Type09::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 7>> {
            match type_code {
                0x08 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type08_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x09 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type09_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Universal (A5-3F)
    pub mod func3F {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Radio Link Test (A5-3F-00), case 0
        #[derive(Clone, Copy)]
        pub struct Type00Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type00Case0PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw MSG_ID value.
            pub fn get_msg_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(29, 2)
            }
            /// Get the MSG_ID value.
            pub fn get_msg_id(&self) -> Option<Type00Case0PropMsg_id> {
                let raw_value = self.get_msg_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw MSG-Source value.
            pub fn get_msg_source_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the MSG-Source value.
            pub fn get_msg_source(&self) -> Option<Type00Case0PropMsgSource> {
                let raw_value = self.get_msg_source_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case0")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("msg_id", &self.get_msg_id())
                    .field("msg_source", &self.get_msg_source())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case0PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case0PropMsg_id {
            Value2 = 2,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case0PropMsgSource {
            RltMaster = false,
            _Other(bool),
        }
        /// Radio Link Test (A5-3F-00), case 1
        #[derive(Clone, Copy)]
        pub struct Type00Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type00Case1PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Sub-Telegram Counter value.
            pub fn get_sub_telegram_counter_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 2)
            }
            /// Get the Sub-Telegram Counter value.
            pub fn get_sub_telegram_counter(&self) -> Option<Type00Case1PropSubTelegramCounter> {
                let raw_value = self.get_sub_telegram_counter_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw RSSI Level in dBm value.
            pub fn get_rssi_level_in_dbm_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(2, 6)
            }
            /// Get the RSSI Level in dBm value.
            pub fn get_rssi_level_in_dbm(&self) -> Option<Type00Case1PropRssiLevelInDbm> {
                let raw_value = self.get_rssi_level_in_dbm_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Sub-Telegram Counter/RSSI Level in dBm value.
            pub fn get_sub_telegram_counter_rssi_level_in_dbm_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Sub-Telegram Counter/RSSI Level in dBm value.
            pub fn get_sub_telegram_counter_rssi_level_in_dbm(&self) -> Option<Type00Case1PropSubTelegramCounterRssiLevelInDbm> {
                let raw_value = self.get_sub_telegram_counter_rssi_level_in_dbm_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Sub-Telegram Counter/RSSI Level in dBm 1 value.
            pub fn get_sub_telegram_counter_rssi_level_in_dbm_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Sub-Telegram Counter/RSSI Level in dBm 1 value.
            pub fn get_sub_telegram_counter_rssi_level_in_dbm_1(&self) -> Option<Type00Case1PropSubTelegramCounterRssiLevelInDbm1> {
                let raw_value = self.get_sub_telegram_counter_rssi_level_in_dbm_1_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw RSSI Level in dBm 1 value.
            pub fn get_rssi_level_in_dbm_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the RSSI Level in dBm 1 value.
            pub fn get_rssi_level_in_dbm_1(&self) -> Option<Type00Case1PropRssiLevelInDbm1> {
                let raw_value = self.get_rssi_level_in_dbm_1_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw MSG_ID value.
            pub fn get_msg_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(29, 2)
            }
            /// Get the MSG_ID value.
            pub fn get_msg_id(&self) -> Option<Type00Case1PropMsg_id> {
                let raw_value = self.get_msg_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw MSG-Source value.
            pub fn get_msg_source_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the MSG-Source value.
            pub fn get_msg_source(&self) -> Option<Type00Case1PropMsgSource> {
                let raw_value = self.get_msg_source_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case1")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("sub_telegram_counter", &self.get_sub_telegram_counter())
                    .field("rssi_level_in_dbm", &self.get_rssi_level_in_dbm())
                    .field("sub_telegram_counter_rssi_level_in_dbm", &self.get_sub_telegram_counter_rssi_level_in_dbm())
                    .field("sub_telegram_counter_rssi_level_in_dbm_1", &self.get_sub_telegram_counter_rssi_level_in_dbm_1())
                    .field("rssi_level_in_dbm_1", &self.get_rssi_level_in_dbm_1())
                    .field("msg_id", &self.get_msg_id())
                    .field("msg_source", &self.get_msg_source())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropSubTelegramCounter {
            NotSupported = 0,
            _1SubTelegram = 1,
            _2SubTelegram = 2,
            _3SubTelegram = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropRssiLevelInDbm {
            NotSupported = 0,
            Minus31 = 1,
            Minus32 = 2,
            Minus93 = 63,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropSubTelegramCounterRssiLevelInDbm {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropSubTelegramCounterRssiLevelInDbm1 {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropRssiLevelInDbm1 {
            NotSupported = 0,
            Minus31 = 1,
            Minus32Minus37 = 2,
            Minus38Minus43 = 3,
            Minus44Minus49 = 4,
            Minus50Minus55 = 5,
            Minus56Minus61 = 6,
            Minus62Minus67 = 7,
            Minus68Minus73 = 8,
            Minus74Minus79 = 9,
            Minus80Minus85 = 10,
            Minus92 = 11,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropMsg_id {
            Value2 = 2,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropMsgSource {
            RltSlave = true,
            _Other(bool),
        }
        /// Radio Link Test (A5-3F-00), case 2
        #[derive(Clone, Copy)]
        pub struct Type00Case2<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case2<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(4, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type00Case2PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw RLT MSG-Counter MSB value.
            pub fn get_rlt_msg_counter_msb_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the RLT MSG-Counter MSB value.
            pub fn get_rlt_msg_counter_msb(&self) -> Option<Type00Case2PropRltMsgCounterMsb> {
                let raw_value = self.get_rlt_msg_counter_msb_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw RLT MSG-Counter LSB value.
            pub fn get_rlt_msg_counter_lsb_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(5, 2)
            }
            /// Get the RLT MSG-Counter LSB value.
            pub fn get_rlt_msg_counter_lsb(&self) -> Option<Type00Case2PropRltMsgCounterLsb> {
                let raw_value = self.get_rlt_msg_counter_lsb_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw MSG-Source value.
            pub fn get_msg_source_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(7, 1)
            }
            /// Get the MSG-Source value.
            pub fn get_msg_source(&self) -> Option<Type00Case2PropMsgSource> {
                let raw_value = self.get_msg_source_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case2<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case2")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("rlt_msg_counter_msb", &self.get_rlt_msg_counter_msb())
                    .field("rlt_msg_counter_lsb", &self.get_rlt_msg_counter_lsb())
                    .field("msg_source", &self.get_msg_source())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropRltMsgCounterMsb {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropRltMsgCounterLsb {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropMsgSource {
            RltMaster = false,
            _Other(bool),
        }
        /// Radio Link Test (A5-3F-00), case 3
        #[derive(Clone, Copy)]
        pub struct Type00Case3<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case3<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(4, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type00Case3PropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw RLT MSG-Counter MSB value.
            pub fn get_rlt_msg_counter_msb_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the RLT MSG-Counter MSB value.
            pub fn get_rlt_msg_counter_msb(&self) -> Option<Type00Case3PropRltMsgCounterMsb> {
                let raw_value = self.get_rlt_msg_counter_msb_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw RLT MSG-Counter LSB value.
            pub fn get_rlt_msg_counter_lsb_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(5, 2)
            }
            /// Get the RLT MSG-Counter LSB value.
            pub fn get_rlt_msg_counter_lsb(&self) -> Option<Type00Case3PropRltMsgCounterLsb> {
                let raw_value = self.get_rlt_msg_counter_lsb_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw MSG-Source value.
            pub fn get_msg_source_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(7, 1)
            }
            /// Get the MSG-Source value.
            pub fn get_msg_source(&self) -> Option<Type00Case3PropMsgSource> {
                let raw_value = self.get_msg_source_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case3<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case3")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("rlt_msg_counter_msb", &self.get_rlt_msg_counter_msb())
                    .field("rlt_msg_counter_lsb", &self.get_rlt_msg_counter_lsb())
                    .field("msg_source", &self.get_msg_source())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case3PropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case3PropRltMsgCounterMsb {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case3PropRltMsgCounterLsb {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case3PropMsgSource {
            RltSlave = true,
            _Other(bool),
        }
        /// Universal (A5-3F-7F)
        #[derive(Clone, Copy)]
        pub struct Type7F<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type7F<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LRN Bit value.
            pub fn get_lrn_bit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(28, 1)
            }
            /// Get the LRN Bit value.
            pub fn get_lrn_bit(&self) -> Option<Type7FPropLrnBit> {
                let raw_value = self.get_lrn_bit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw undefined value.
            pub fn get_undefined_raw(&self) -> Option<u32> {
                self.reversed_bytes.u32_from_bits(0, 28)
            }

            /// Get the raw undefined 1 value.
            pub fn get_undefined_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(29, 3)
            }
        }
        impl<'b> ::core::fmt::Debug for Type7F<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type7F")
                    .field("lrn_bit", &self.get_lrn_bit())
                    .field("undefined", &self.get_undefined_raw())
                    .field("undefined_1", &self.get_undefined_1_raw())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type7FPropLrnBit {
            TeachInTelegram = false,
            DataTelegram = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func3F<'b> {
        Type00Case0(func3F::Type00Case0<'b>),
        Type00Case1(func3F::Type00Case1<'b>),
        Type00Case2(func3F::Type00Case2<'b>),
        Type00Case3(func3F::Type00Case3<'b>),
        Type7F(func3F::Type7F<'b>),
    }
    impl<'b> Func3F<'b> {
        pub fn type00_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 4> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type00Case0(
                func3F::Type00Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case1(
                func3F::Type00Case1::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case2(
                func3F::Type00Case2::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case3(
                func3F::Type00Case3::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type7F_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type7F(
                func3F::Type7F::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 4>> {
            match type_code {
                0x00 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type00_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x7F => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type7F_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RorgA5<'b> {
    Func02(rorgA5::Func02<'b>),
    Func06(rorgA5::Func06<'b>),
    Func07(rorgA5::Func07<'b>),
    Func08(rorgA5::Func08<'b>),
    Func09(rorgA5::Func09<'b>),
    Func11(rorgA5::Func11<'b>),
    Func12(rorgA5::Func12<'b>),
    Func13(rorgA5::Func13<'b>),
    Func14(rorgA5::Func14<'b>),
    Func20(rorgA5::Func20<'b>),
    Func30(rorgA5::Func30<'b>),
    Func37(rorgA5::Func37<'b>),
    Func38(rorgA5::Func38<'b>),
    Func3F(rorgA5::Func3F<'b>),
}
impl<'b> RorgA5<'b> {
    pub fn from_reversed_bytes(func_code: u8, type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 7>> {
        match func_code {
            0x02 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgA5::Func02::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func02(*f))
                        .peekable()
                ))
            },
            0x06 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgA5::Func06::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func06(*f))
                        .peekable()
                ))
            },
            0x07 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgA5::Func07::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func07(*f))
                        .peekable()
                ))
            },
            0x08 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgA5::Func08::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func08(*f))
                        .peekable()
                ))
            },
            0x09 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgA5::Func09::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func09(*f))
                        .peekable()
                ))
            },
            0x11 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgA5::Func11::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func11(*f))
                        .peekable()
                ))
            },
            0x12 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgA5::Func12::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func12(*f))
                        .peekable()
                ))
            },
            0x13 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgA5::Func13::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func13(*f))
                        .peekable()
                ))
            },
            0x14 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgA5::Func14::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func14(*f))
                        .peekable()
                ))
            },
            0x20 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgA5::Func20::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func20(*f))
                        .peekable()
                ))
            },
            0x30 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgA5::Func30::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func30(*f))
                        .peekable()
                ))
            },
            0x37 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgA5::Func37::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func37(*f))
                        .peekable()
                ))
            },
            0x38 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgA5::Func38::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func38(*f))
                        .peekable()
                ))
            },
            0x3F => {
                Some(MaxArray::from_iter_or_panic(
                    rorgA5::Func3F::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func3F(*f))
                        .peekable()
                ))
            },
            _ => None,
        }
    }
}
/// VLD Telegram (D2)
#[allow(non_snake_case)]
pub mod rorgD2 {
    #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
    /// Room Control Panel (RCP) (D2-00)
    pub mod func00 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// RCP with Temperature Measurement and Display (BI-DIR) (D2-00-01), case 0
        #[derive(Clone, Copy)]
        pub struct Type01Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw MsgId value.
            pub fn get_msgid_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(5, 3)
            }
            /// Get the MsgId value.
            pub fn get_msgid(&self) -> Option<Type01Case0PropMsgid> {
                let raw_value = self.get_msgid_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw User Action value.
            pub fn get_user_action_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(11, 5)
            }
            /// Get the User Action value.
            pub fn get_user_action(&self) -> Option<Type01Case0PropUserAction> {
                let raw_value = self.get_user_action_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw ConfigValid value.
            pub fn get_configvalid_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(8, 1)
            }
            /// Get the ConfigValid value.
            pub fn get_configvalid(&self) -> Option<Type01Case0PropConfigvalid> {
                let raw_value = self.get_configvalid_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01Case0")
                    .field("msgid", &self.get_msgid())
                    .field("user_action", &self.get_user_action())
                    .field("configvalid", &self.get_configvalid())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case0PropMsgid {
            MessageId = 1,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case0PropUserAction {
            NotUsed = 0,
            Presence = 1,
            TemperatureSetPointDownOr = 2,
            NotUsed1 = 3,
            NotUsed2 = 4,
            TemperatureSetPointUpOr = 5,
            Fan = 6,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case0PropConfigvalid {
            ConfigurationDataNotValidEGNeverReceivedMessageOfTypeE = false,
            ConfigurationDataValid = true,
            _Other(bool),
        }
        /// RCP with Temperature Measurement and Display (BI-DIR) (D2-00-01), case 1
        #[derive(Clone, Copy)]
        pub struct Type01Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw MsgId value.
            pub fn get_msgid_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(5, 3)
            }
            /// Get the MsgId value.
            pub fn get_msgid(&self) -> Option<Type01Case1PropMsgid> {
                let raw_value = self.get_msgid_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw MoreData value.
            pub fn get_moredata_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(4, 1)
            }
            /// Get the MoreData value.
            pub fn get_moredata(&self) -> Option<Type01Case1PropMoredata> {
                let raw_value = self.get_moredata_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Fan value.
            pub fn get_fan_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(1, 3)
            }
            /// Get the Fan value.
            pub fn get_fan(&self) -> Option<Type01Case1PropFan> {
                let raw_value = self.get_fan_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Fan manual value.
            pub fn get_fan_manual_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(0, 1)
            }
            /// Get the Fan manual value.
            pub fn get_fan_manual(&self) -> Option<Type01Case1PropFanManual> {
                let raw_value = self.get_fan_manual_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Figure A Type value.
            pub fn get_figure_a_type_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(11, 5)
            }
            /// Get the Figure A Type value.
            pub fn get_figure_a_type(&self) -> Option<Type01Case1PropFigureAType> {
                let raw_value = self.get_figure_a_type_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Presence value.
            pub fn get_presence_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 3)
            }
            /// Get the Presence value.
            pub fn get_presence(&self) -> Option<Type01Case1PropPresence> {
                let raw_value = self.get_presence_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Figure A Value value.
            pub fn get_figure_a_value_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(16, 16)
            }
            /// Get the Figure A Value value.
            pub fn get_figure_a_value(&self) -> Option<Type01Case1PropFigureAValue> {
                let raw_value = self.get_figure_a_value_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Heating value.
            pub fn get_heating_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(39, 1)
            }
            /// Get the Heating value.
            pub fn get_heating(&self) -> Option<Type01Case1PropHeating> {
                let raw_value = self.get_heating_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Cooling value.
            pub fn get_cooling_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(38, 1)
            }
            /// Get the Cooling value.
            pub fn get_cooling(&self) -> Option<Type01Case1PropCooling> {
                let raw_value = self.get_cooling_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Dew-Point value.
            pub fn get_dew_point_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(37, 1)
            }
            /// Get the Dew-Point value.
            pub fn get_dew_point(&self) -> Option<Type01Case1PropDewPoint> {
                let raw_value = self.get_dew_point_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Window value.
            pub fn get_window_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(36, 1)
            }
            /// Get the Window value.
            pub fn get_window(&self) -> Option<Type01Case1PropWindow> {
                let raw_value = self.get_window_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw User Notification value.
            pub fn get_user_notification_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(35, 1)
            }
            /// Get the User Notification value.
            pub fn get_user_notification(&self) -> Option<Type01Case1PropUserNotification> {
                let raw_value = self.get_user_notification_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01Case1")
                    .field("msgid", &self.get_msgid())
                    .field("moredata", &self.get_moredata())
                    .field("fan", &self.get_fan())
                    .field("fan_manual", &self.get_fan_manual())
                    .field("figure_a_type", &self.get_figure_a_type())
                    .field("presence", &self.get_presence())
                    .field("figure_a_value", &self.get_figure_a_value())
                    .field("heating", &self.get_heating())
                    .field("cooling", &self.get_cooling())
                    .field("dew_point", &self.get_dew_point())
                    .field("window", &self.get_window())
                    .field("user_notification", &self.get_user_notification())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case1PropMsgid {
            MessageId = 2,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropMoredata {
            NoMoreData = false,
            MoreDataWillFollowAfterT2 = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case1PropFan {
            DoNotDisplay = 0,
            SpeedLevel0 = 1,
            SpeedLevel1 = 2,
            SpeedLevel2 = 3,
            SpeedLevel3 = 4,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropFanManual {
            Auto = false,
            FanManual = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case1PropFigureAType {
            DoNotDisplay = 0,
            RoomTemperature = 1,
            RoomTemperature1 = 2,
            NominalTemperature = 3,
            NominalTemperature1 = 4,
            DeltaTemperatureSetPoint = 5,
            DeltaTemperatureSetPoint1 = 6,
            DeltaTemperatureSetPointGraphic = 7,
            Time0000To235924h = 8,
            Time0000To1159Am = 9,
            Time0000To1159Pm = 10,
            Date0101To3112DdMm = 11,
            Date0101To1231MmDd = 12,
            IlluminationLinear0To9999 = 13,
            Percentage0To100 = 14,
            PartsPerMillion0To9999 = 15,
            RelativeHumidity0To100 = 16,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case1PropPresence {
            DoNotDisplay = 0,
            Present = 1,
            NotPresent = 2,
            NightTimeReduction = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u16, derive_compare = "as_int")]
        pub enum Type01Case1PropFigureAValue {
            _09999 = 13,
            _099991 = 15,
            _Other(u16),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropHeating {
            Off = false,
            On = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropCooling {
            Off = false,
            On = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropDewPoint {
            Warning = false,
            NoWarning = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropWindow {
            Closed = false,
            Opened = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case1PropUserNotification {
            Off = false,
            On = true,
            _Other(bool),
        }
        /// RCP with Temperature Measurement and Display (BI-DIR) (D2-00-01), case 2
        #[derive(Clone, Copy)]
        pub struct Type01Case2<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01Case2<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw MsgId value.
            pub fn get_msgid_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(5, 3)
            }
            /// Get the MsgId value.
            pub fn get_msgid(&self) -> Option<Type01Case2PropMsgid> {
                let raw_value = self.get_msgid_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Fan value.
            pub fn get_fan_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(1, 3)
            }
            /// Get the Fan value.
            pub fn get_fan(&self) -> Option<Type01Case2PropFan> {
                let raw_value = self.get_fan_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Set Point A Type value.
            pub fn get_set_point_a_type_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(11, 5)
            }
            /// Get the Set Point A Type value.
            pub fn get_set_point_a_type(&self) -> Option<Type01Case2PropSetPointAType> {
                let raw_value = self.get_set_point_a_type_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Presence value.
            pub fn get_presence_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 3)
            }
            /// Get the Presence value.
            pub fn get_presence(&self) -> Option<Type01Case2PropPresence> {
                let raw_value = self.get_presence_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Set Point A Value value.
            pub fn get_set_point_a_value_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(16, 16)
            }
            /// Get the Set Point A Value value in units of °.
            pub fn get_set_point_a_value(&self) -> Option<f64> {
                let raw_value = self.get_set_point_a_value_raw()? as f64;
                Some(range_scale(raw_value, -1270.0, 1270.0, -12.7, 12.7))
            }
        }
        impl<'b> ::core::fmt::Debug for Type01Case2<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01Case2")
                    .field("msgid", &self.get_msgid())
                    .field("fan", &self.get_fan())
                    .field("set_point_a_type", &self.get_set_point_a_type())
                    .field("presence", &self.get_presence())
                    .field("set_point_a_value", &self.get_set_point_a_value())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case2PropMsgid {
            MessageId = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case2PropFan {
            NoChange = 0,
            SpeedLevel0 = 1,
            SpeedLevel1 = 2,
            SpeedLevel2 = 3,
            SpeedLevel3 = 4,
            SpeedLevelAuto = 5,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case2PropSetPointAType {
            NoChange = 0,
            TemperatureSetPointDegrees = 5,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case2PropPresence {
            NoChange = 0,
            Present = 1,
            NotPresent = 2,
            NightTimeReduction = 3,
            _Other(u8),
        }
        /// RCP with Temperature Measurement and Display (BI-DIR) (D2-00-01), case 3
        #[derive(Clone, Copy)]
        pub struct Type01Case3<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01Case3<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw MsgId value.
            pub fn get_msgid_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(5, 3)
            }
            /// Get the MsgId value.
            pub fn get_msgid(&self) -> Option<Type01Case3PropMsgid> {
                let raw_value = self.get_msgid_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Channel A Value value.
            pub fn get_channel_a_value_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Channel A Value value in units of °.
            pub fn get_channel_a_value(&self) -> Option<f64> {
                let raw_value = self.get_channel_a_value_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 4000.0, 0.0, 40.0))
            }

            /// Get the raw Channel A Value 1 value.
            pub fn get_channel_a_value_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(20, 4)
            }

            /// Get the raw Channel A Type value.
            pub fn get_channel_a_type_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 4)
            }
            /// Get the Channel A Type value.
            pub fn get_channel_a_type(&self) -> Option<Type01Case3PropChannelAType> {
                let raw_value = self.get_channel_a_type_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01Case3<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01Case3")
                    .field("msgid", &self.get_msgid())
                    .field("channel_a_value", &self.get_channel_a_value())
                    .field("channel_a_value_1", &self.get_channel_a_value_1_raw())
                    .field("channel_a_type", &self.get_channel_a_type())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case3PropMsgid {
            MessageId = 4,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case3PropChannelAType {
            TemperatureDegreesC = 0,
            MeasurementResultNotValid = 15,
            _Other(u8),
        }
        /// RCP with Temperature Measurement and Display (BI-DIR) (D2-00-01), case 4
        #[derive(Clone, Copy)]
        pub struct Type01Case4<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01Case4<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw MsgId value.
            pub fn get_msgid_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(5, 3)
            }
            /// Get the MsgId value.
            pub fn get_msgid(&self) -> Option<Type01Case4PropMsgid> {
                let raw_value = self.get_msgid_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw MoreData value.
            pub fn get_moredata_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(4, 1)
            }
            /// Get the MoreData value.
            pub fn get_moredata(&self) -> Option<Type01Case4PropMoredata> {
                let raw_value = self.get_moredata_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Set Point Range Limit value.
            pub fn get_set_point_range_limit_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(9, 7)
            }
            /// Get the Set Point Range Limit value.
            pub fn get_set_point_range_limit(&self) -> Option<Type01Case4PropSetPointRangeLimit> {
                let raw_value = self.get_set_point_range_limit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Set PointSteps value.
            pub fn get_set_pointsteps_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(17, 7)
            }
            /// Get the Set PointSteps value.
            pub fn get_set_pointsteps(&self) -> Option<Type01Case4PropSetPointsteps> {
                let raw_value = self.get_set_pointsteps_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature Measurement Timing value.
            pub fn get_temperature_measurement_timing_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 4)
            }
            /// Get the Temperature Measurement Timing value.
            pub fn get_temperature_measurement_timing(&self) -> Option<Type01Case4PropTemperatureMeasurementTiming> {
                let raw_value = self.get_temperature_measurement_timing_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature Measurement Timing 1 value.
            pub fn get_temperature_measurement_timing_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(38, 2)
            }

            /// Get the raw Fan value.
            pub fn get_fan_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(35, 3)
            }
            /// Get the Fan value.
            pub fn get_fan(&self) -> Option<Type01Case4PropFan> {
                let raw_value = self.get_fan_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Presence value.
            pub fn get_presence_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(32, 3)
            }
            /// Get the Presence value.
            pub fn get_presence(&self) -> Option<Type01Case4PropPresence> {
                let raw_value = self.get_presence_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Keep Alive Timing value.
            pub fn get_keep_alive_timing_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(45, 3)
            }
            /// Get the Keep Alive Timing value.
            pub fn get_keep_alive_timing(&self) -> Option<Type01Case4PropKeepAliveTiming> {
                let raw_value = self.get_keep_alive_timing_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Significant Temperature Difference value.
            pub fn get_significant_temperature_difference_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(40, 4)
            }
        }
        impl<'b> ::core::fmt::Debug for Type01Case4<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01Case4")
                    .field("msgid", &self.get_msgid())
                    .field("moredata", &self.get_moredata())
                    .field("set_point_range_limit", &self.get_set_point_range_limit())
                    .field("set_pointsteps", &self.get_set_pointsteps())
                    .field("temperature_measurement_timing", &self.get_temperature_measurement_timing())
                    .field("temperature_measurement_timing_1", &self.get_temperature_measurement_timing_1_raw())
                    .field("fan", &self.get_fan())
                    .field("presence", &self.get_presence())
                    .field("keep_alive_timing", &self.get_keep_alive_timing())
                    .field("significant_temperature_difference", &self.get_significant_temperature_difference_raw())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case4PropMsgid {
            MessageId = 5,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01Case4PropMoredata {
            NoMoreData = false,
            MoreDataWillFollowAfterT2 = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case4PropSetPointRangeLimit {
            SetPointDisabled = 0,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case4PropSetPointsteps {
            SetPointDisabled = 0,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case4PropTemperatureMeasurementTiming {
            TemperatureMeasurementDisabled = 0,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case4PropFan {
            FanSpeedDisabled = 0,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case4PropPresence {
            PresenceDisabled = 0,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case4PropKeepAliveTiming {
            TransmissionOfMeasurementResultWithEachTemperatureMeasurement = 0,
            _Other(u8),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func00<'b> {
        Type01Case0(func00::Type01Case0<'b>),
        Type01Case1(func00::Type01Case1<'b>),
        Type01Case2(func00::Type01Case2<'b>),
        Type01Case3(func00::Type01Case3<'b>),
        Type01Case4(func00::Type01Case4<'b>),
    }
    impl<'b> Func00<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 5> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01Case0(
                func00::Type01Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type01Case1(
                func00::Type01Case1::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type01Case2(
                func00::Type01Case2::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type01Case3(
                func00::Type01Case3::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type01Case4(
                func00::Type01Case4::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 5>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Multichannel Temperature Sensor (D2-0A)
    pub mod func0A {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Type 0x00 (D2-0A-00)
        #[derive(Clone, Copy)]
        pub struct Type00<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Battery Life value.
            pub fn get_battery_life_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(0, 1)
            }
            /// Get the Battery Life value.
            pub fn get_battery_life(&self) -> Option<Type00PropBatteryLife> {
                let raw_value = self.get_battery_life_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Channel 1 value.
            pub fn get_channel_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Channel 1 value.
            pub fn get_channel_1(&self) -> Option<Type00PropChannel1> {
                let raw_value = self.get_channel_1_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Channel 2 value.
            pub fn get_channel_2_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Channel 2 value.
            pub fn get_channel_2(&self) -> Option<Type00PropChannel2> {
                let raw_value = self.get_channel_2_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Channel 3 value.
            pub fn get_channel_3_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 8)
            }
            /// Get the Channel 3 value.
            pub fn get_channel_3(&self) -> Option<Type00PropChannel3> {
                let raw_value = self.get_channel_3_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00")
                    .field("battery_life", &self.get_battery_life())
                    .field("channel_1", &self.get_channel_1())
                    .field("channel_2", &self.get_channel_2())
                    .field("channel_3", &self.get_channel_3())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00PropBatteryLife {
            Ok = false,
            Low = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropChannel1 {
            Fault = 254,
            Disconnected = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropChannel2 {
            Fault = 254,
            Disconnected = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropChannel3 {
            Fault = 254,
            Disconnected = 255,
            _Other(u8),
        }
        /// Type 0x01 (D2-0A-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Battery Life value.
            pub fn get_battery_life_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(0, 1)
            }
            /// Get the Battery Life value.
            pub fn get_battery_life(&self) -> Option<Type01PropBatteryLife> {
                let raw_value = self.get_battery_life_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Channel 1 value.
            pub fn get_channel_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Channel 1 value.
            pub fn get_channel_1(&self) -> Option<Type01PropChannel1> {
                let raw_value = self.get_channel_1_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Channel 2 value.
            pub fn get_channel_2_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Channel 2 value.
            pub fn get_channel_2(&self) -> Option<Type01PropChannel2> {
                let raw_value = self.get_channel_2_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Channel 3 value.
            pub fn get_channel_3_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 8)
            }
            /// Get the Channel 3 value.
            pub fn get_channel_3(&self) -> Option<Type01PropChannel3> {
                let raw_value = self.get_channel_3_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("battery_life", &self.get_battery_life())
                    .field("channel_1", &self.get_channel_1())
                    .field("channel_2", &self.get_channel_2())
                    .field("channel_3", &self.get_channel_3())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropBatteryLife {
            Ok = false,
            Low = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropChannel1 {
            Fault = 254,
            Disconnected = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropChannel2 {
            Fault = 254,
            Disconnected = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropChannel3 {
            Fault = 254,
            Disconnected = 255,
            _Other(u8),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func0A<'b> {
        Type00(func0A::Type00<'b>),
        Type01(func0A::Type01<'b>),
    }
    impl<'b> Func0A<'b> {
        pub fn type00_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type00(
                func0A::Type00::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func0A::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x00 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type00_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Occupancy (D2-15)
    pub mod func15 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// People Activity Counter (D2-15-00)
        #[derive(Clone, Copy)]
        pub struct Type00<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Presence value.
            pub fn get_presence_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 2)
            }
            /// Get the Presence value.
            pub fn get_presence(&self) -> Option<Type00PropPresence> {
                let raw_value = self.get_presence_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy Storage value.
            pub fn get_energy_storage_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(2, 2)
            }
            /// Get the Energy Storage value.
            pub fn get_energy_storage(&self) -> Option<Type00PropEnergyStorage> {
                let raw_value = self.get_energy_storage_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Pir Update Rate value.
            pub fn get_pir_update_rate_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Pir Update Rate value in units of s.
            pub fn get_pir_update_rate(&self) -> Option<f64> {
                let raw_value = self.get_pir_update_rate_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 15.0, 1.0, 16.0))
            }

            /// Get the raw Pir Update Counter value.
            pub fn get_pir_update_counter_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(8, 16)
            }
            /// Get the Pir Update Counter value.
            pub fn get_pir_update_counter(&self) -> Option<f64> {
                let raw_value = self.get_pir_update_counter_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 65535.0, 0.0, 65535.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type00<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00")
                    .field("presence", &self.get_presence())
                    .field("energy_storage", &self.get_energy_storage())
                    .field("pir_update_rate", &self.get_pir_update_rate())
                    .field("pir_update_counter", &self.get_pir_update_counter())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropPresence {
            Present = 0,
            NotPresent = 1,
            NotDetectable = 2,
            PresenceDetectionError = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropEnergyStorage {
            High = 0,
            Medium = 1,
            Low = 2,
            Critical = 3,
            _Other(u8),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func15<'b> {
        Type00(func15::Type00<'b>),
    }
    impl<'b> Func15<'b> {
        pub fn type00_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type00(
                func15::Type00::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x00 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type00_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Automated Meter Reading Gateway (D2-31)
    pub mod func31 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Type 0x00 (D2-31-00), case 0
        #[derive(Clone, Copy)]
        pub struct Type00Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Report measurement value.
            pub fn get_report_measurement_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Report measurement value.
            pub fn get_report_measurement(&self) -> Option<Type00Case0PropReportMeasurement> {
                let raw_value = self.get_report_measurement_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case0PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter bus type value.
            pub fn get_meter_bus_type_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(9, 2)
            }
            /// Get the Meter bus type value.
            pub fn get_meter_bus_type(&self) -> Option<Type00Case0PropMeterBusType> {
                let raw_value = self.get_meter_bus_type_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter channel index value.
            pub fn get_meter_channel_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(11, 5)
            }
            /// Get the Meter channel index value in units of 1.
            pub fn get_meter_channel_index(&self) -> Option<f64> {
                let raw_value = self.get_meter_channel_index_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 30.0, 0.0, 30.0))
            }

            /// Get the raw Meter 1 units value.
            pub fn get_meter_1_units_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(18, 3)
            }
            /// Get the Meter 1 units value.
            pub fn get_meter_1_units(&self) -> Option<Type00Case0PropMeter1Units> {
                let raw_value = self.get_meter_1_units_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter 2 units value.
            pub fn get_meter_2_units_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(21, 3)
            }
            /// Get the Meter 2 units value.
            pub fn get_meter_2_units(&self) -> Option<Type00Case0PropMeter2Units> {
                let raw_value = self.get_meter_2_units_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Primary Address value.
            pub fn get_primary_address_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 8)
            }
            /// Get the Primary Address value in units of 1.
            pub fn get_primary_address(&self) -> Option<f64> {
                let raw_value = self.get_primary_address_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 250.0, 1.0, 250.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case0")
                    .field("report_measurement", &self.get_report_measurement())
                    .field("command_id", &self.get_command_id())
                    .field("meter_bus_type", &self.get_meter_bus_type())
                    .field("meter_channel_index", &self.get_meter_channel_index())
                    .field("meter_1_units", &self.get_meter_1_units())
                    .field("meter_2_units", &self.get_meter_2_units())
                    .field("primary_address", &self.get_primary_address())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case0PropReportMeasurement {
            NoAutoReporting = 0,
            Min1SInterval = 1,
            Min3SInterval = 2,
            Min10SInterval = 3,
            Min30SInterval = 4,
            Min100SInterval = 5,
            Min300SInterval = 6,
            Min1000SInterval = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case0PropCommandId {
            Id06 = 6,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case0PropMeterBusType {
            Reserved = 0,
            Mbus = 1,
            S0 = 2,
            D0 = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case0PropMeter1Units {
            NoReadingUnconfigured = 0,
            CurrentValueWAccumulatedValueKwh = 1,
            CurrentValueWAccumulatedValueWh = 2,
            AccumulatedValueKwhOnly = 3,
            CurrentValueM3HAccumulatedValueM3 = 4,
            CurrentValueDm3HAccumulatedValueDm3 = 5,
            AccumulatedValueM3Only = 6,
            DigitalCounter = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case0PropMeter2Units {
            NoReadingUnconfigured = 0,
            CurrentValueWAccumulatedValueKwh = 1,
            CurrentValueWAccumulatedValueWh = 2,
            AccumulatedValueKwhOnly = 3,
            CurrentValueM3HAccumulatedValueM3 = 4,
            CurrentValueDm3HAccumulatedValueDm3 = 5,
            AccumulatedValueM3Only = 6,
            DigitalCounter = 7,
            _Other(u8),
        }
        /// Type 0x00 (D2-31-00), case 1
        #[derive(Clone, Copy)]
        pub struct Type00Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Report measurement value.
            pub fn get_report_measurement_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Report measurement value.
            pub fn get_report_measurement(&self) -> Option<Type00Case1PropReportMeasurement> {
                let raw_value = self.get_report_measurement_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case1PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter bus type value.
            pub fn get_meter_bus_type_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(9, 2)
            }
            /// Get the Meter bus type value.
            pub fn get_meter_bus_type(&self) -> Option<Type00Case1PropMeterBusType> {
                let raw_value = self.get_meter_bus_type_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter channel index value.
            pub fn get_meter_channel_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(11, 5)
            }
            /// Get the Meter channel index value in units of 1.
            pub fn get_meter_channel_index(&self) -> Option<f64> {
                let raw_value = self.get_meter_channel_index_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 30.0, 0.0, 30.0))
            }

            /// Get the raw Meter 1 units value.
            pub fn get_meter_1_units_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(18, 3)
            }
            /// Get the Meter 1 units value.
            pub fn get_meter_1_units(&self) -> Option<Type00Case1PropMeter1Units> {
                let raw_value = self.get_meter_1_units_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter 2 units value.
            pub fn get_meter_2_units_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(21, 3)
            }
            /// Get the Meter 2 units value.
            pub fn get_meter_2_units(&self) -> Option<Type00Case1PropMeter2Units> {
                let raw_value = self.get_meter_2_units_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Factor of number of pulses value.
            pub fn get_factor_of_number_of_pulses_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 2)
            }
            /// Get the Factor of number of pulses value.
            pub fn get_factor_of_number_of_pulses(&self) -> Option<Type00Case1PropFactorOfNumberOfPulses> {
                let raw_value = self.get_factor_of_number_of_pulses_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Number of pulses value.
            pub fn get_number_of_pulses_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(26, 14)
            }
            /// Get the Number of pulses value.
            pub fn get_number_of_pulses(&self) -> Option<Type00Case1PropNumberOfPulses> {
                let raw_value = self.get_number_of_pulses_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Preset value value.
            pub fn get_preset_value_raw(&self) -> Option<u32> {
                self.reversed_bytes.u32_from_bits(40, 32)
            }
            /// Get the Preset value value.
            pub fn get_preset_value(&self) -> Option<Type00Case1PropPresetValue> {
                let raw_value = self.get_preset_value_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case1")
                    .field("report_measurement", &self.get_report_measurement())
                    .field("command_id", &self.get_command_id())
                    .field("meter_bus_type", &self.get_meter_bus_type())
                    .field("meter_channel_index", &self.get_meter_channel_index())
                    .field("meter_1_units", &self.get_meter_1_units())
                    .field("meter_2_units", &self.get_meter_2_units())
                    .field("factor_of_number_of_pulses", &self.get_factor_of_number_of_pulses())
                    .field("number_of_pulses", &self.get_number_of_pulses())
                    .field("preset_value", &self.get_preset_value())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropReportMeasurement {
            NoAutoReporting = 0,
            Min1SInterval = 1,
            Min3SInterval = 2,
            Min10SInterval = 3,
            Min30SInterval = 4,
            Min100SInterval = 5,
            Min300SInterval = 6,
            Min1000SInterval = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropCommandId {
            Id06 = 6,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropMeterBusType {
            Reserved = 0,
            Mbus = 1,
            S0 = 2,
            D0 = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropMeter1Units {
            NoReadingUnconfigured = 0,
            CurrentValueWAccumulatedValueKwh = 1,
            CurrentValueWAccumulatedValueWh = 2,
            AccumulatedValueKwhOnly = 3,
            CurrentValueM3HAccumulatedValueM3 = 4,
            CurrentValueDm3HAccumulatedValueDm3 = 5,
            AccumulatedValueM3Only = 6,
            DigitalCounter = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropMeter2Units {
            NoReadingUnconfigured = 0,
            CurrentValueWAccumulatedValueKwh = 1,
            CurrentValueWAccumulatedValueWh = 2,
            AccumulatedValueKwhOnly = 3,
            CurrentValueM3HAccumulatedValueM3 = 4,
            CurrentValueDm3HAccumulatedValueDm3 = 5,
            AccumulatedValueM3Only = 6,
            DigitalCounter = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropFactorOfNumberOfPulses {
            _1 = 0,
            _01 = 1,
            _001 = 2,
            _0001 = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u16, derive_compare = "as_int")]
        pub enum Type00Case1PropNumberOfPulses {
            DoNotChangeTheCurrentSettingOfNop = 0,
            _Other(u16),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u32, derive_compare = "as_int")]
        pub enum Type00Case1PropPresetValue {
            DoNotChangeTheCurrentValue = 4294967295,
            _Other(u32),
        }
        /// Type 0x00 (D2-31-00), case 2
        #[derive(Clone, Copy)]
        pub struct Type00Case2<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case2<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Report measurement value.
            pub fn get_report_measurement_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Report measurement value.
            pub fn get_report_measurement(&self) -> Option<Type00Case2PropReportMeasurement> {
                let raw_value = self.get_report_measurement_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case2PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter bus type value.
            pub fn get_meter_bus_type_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(9, 2)
            }
            /// Get the Meter bus type value.
            pub fn get_meter_bus_type(&self) -> Option<Type00Case2PropMeterBusType> {
                let raw_value = self.get_meter_bus_type_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter channel index value.
            pub fn get_meter_channel_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(11, 5)
            }
            /// Get the Meter channel index value in units of 1.
            pub fn get_meter_channel_index(&self) -> Option<f64> {
                let raw_value = self.get_meter_channel_index_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 30.0, 0.0, 30.0))
            }

            /// Get the raw Meter 1 units value.
            pub fn get_meter_1_units_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(18, 3)
            }
            /// Get the Meter 1 units value.
            pub fn get_meter_1_units(&self) -> Option<Type00Case2PropMeter1Units> {
                let raw_value = self.get_meter_1_units_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter 2 units value.
            pub fn get_meter_2_units_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(21, 3)
            }
            /// Get the Meter 2 units value.
            pub fn get_meter_2_units(&self) -> Option<Type00Case2PropMeter2Units> {
                let raw_value = self.get_meter_2_units_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw D0 Protocol value.
            pub fn get_d0_protocol_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 8)
            }
            /// Get the D0 Protocol value.
            pub fn get_d0_protocol(&self) -> Option<Type00Case2PropD0Protocol> {
                let raw_value = self.get_d0_protocol_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case2<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case2")
                    .field("report_measurement", &self.get_report_measurement())
                    .field("command_id", &self.get_command_id())
                    .field("meter_bus_type", &self.get_meter_bus_type())
                    .field("meter_channel_index", &self.get_meter_channel_index())
                    .field("meter_1_units", &self.get_meter_1_units())
                    .field("meter_2_units", &self.get_meter_2_units())
                    .field("d0_protocol", &self.get_d0_protocol())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropReportMeasurement {
            NoAutoReporting = 0,
            Min1SInterval = 1,
            Min3SInterval = 2,
            Min10SInterval = 3,
            Min30SInterval = 4,
            Min100SInterval = 5,
            Min300SInterval = 6,
            Min1000SInterval = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropCommandId {
            Id06 = 6,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropMeterBusType {
            Reserved = 0,
            Mbus = 1,
            S0 = 2,
            D0 = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropMeter1Units {
            NoReadingUnconfigured = 0,
            CurrentValueWAccumulatedValueKwh = 1,
            CurrentValueWAccumulatedValueWh = 2,
            AccumulatedValueKwhOnly = 3,
            CurrentValueM3HAccumulatedValueM3 = 4,
            CurrentValueDm3HAccumulatedValueDm3 = 5,
            AccumulatedValueM3Only = 6,
            DigitalCounter = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropMeter2Units {
            NoReadingUnconfigured = 0,
            CurrentValueWAccumulatedValueKwh = 1,
            CurrentValueWAccumulatedValueWh = 2,
            AccumulatedValueKwhOnly = 3,
            CurrentValueM3HAccumulatedValueM3 = 4,
            CurrentValueDm3HAccumulatedValueDm3 = 5,
            AccumulatedValueM3Only = 6,
            DigitalCounter = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropD0Protocol {
            AutoDetect = 0,
            SmlSmartMessageLanguage = 1,
            DlmsDeviceLanguageMessageSpecification = 2,
            _Other(u8),
        }
        /// Type 0x00 (D2-31-00), case 3
        #[derive(Clone, Copy)]
        pub struct Type00Case3<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case3<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case3PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter bus type value.
            pub fn get_meter_bus_type_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(9, 2)
            }
            /// Get the Meter bus type value.
            pub fn get_meter_bus_type(&self) -> Option<Type00Case3PropMeterBusType> {
                let raw_value = self.get_meter_bus_type_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter channel index value.
            pub fn get_meter_channel_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(11, 5)
            }
            /// Get the Meter channel index value.
            pub fn get_meter_channel_index(&self) -> Option<Type00Case3PropMeterChannelIndex> {
                let raw_value = self.get_meter_channel_index_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case3<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case3")
                    .field("command_id", &self.get_command_id())
                    .field("meter_bus_type", &self.get_meter_bus_type())
                    .field("meter_channel_index", &self.get_meter_channel_index())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case3PropCommandId {
            Id07 = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case3PropMeterBusType {
            Reserved = 0,
            Mbus = 1,
            S0 = 2,
            D0 = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case3PropMeterChannelIndex {
            AllValidChannels = 31,
            _Other(u8),
        }
        /// Type 0x00 (D2-31-00), case 4
        #[derive(Clone, Copy)]
        pub struct Type00Case4<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case4<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Meter status / error value.
            pub fn get_meter_status_error_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(1, 3)
            }
            /// Get the Meter status / error value.
            pub fn get_meter_status_error(&self) -> Option<Type00Case4PropMeterStatusError> {
                let raw_value = self.get_meter_status_error_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case4PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter bus type value.
            pub fn get_meter_bus_type_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(9, 2)
            }
            /// Get the Meter bus type value.
            pub fn get_meter_bus_type(&self) -> Option<Type00Case4PropMeterBusType> {
                let raw_value = self.get_meter_bus_type_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter channel index value.
            pub fn get_meter_channel_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(11, 5)
            }
            /// Get the Meter channel index value in units of 1.
            pub fn get_meter_channel_index(&self) -> Option<f64> {
                let raw_value = self.get_meter_channel_index_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 30.0, 0.0, 30.0))
            }

            /// Get the raw Value selection value.
            pub fn get_value_selection_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(19, 2)
            }
            /// Get the Value selection value.
            pub fn get_value_selection(&self) -> Option<Type00Case4PropValueSelection> {
                let raw_value = self.get_value_selection_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Value unit value.
            pub fn get_value_unit_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(21, 3)
            }
            /// Get the Value unit value.
            pub fn get_value_unit(&self) -> Option<Type00Case4PropValueUnit> {
                let raw_value = self.get_value_unit_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Meter reading value value.
            pub fn get_meter_reading_value_raw(&self) -> Option<u32> {
                self.reversed_bytes.u32_from_bits(24, 32)
            }
            /// Get the Meter reading value value in units of According to VUNIT.
            pub fn get_meter_reading_value(&self) -> Option<f64> {
                let raw_value = self.get_meter_reading_value_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 4294967295.0, 0.0, 4294967295.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case4<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case4")
                    .field("meter_status_error", &self.get_meter_status_error())
                    .field("command_id", &self.get_command_id())
                    .field("meter_bus_type", &self.get_meter_bus_type())
                    .field("meter_channel_index", &self.get_meter_channel_index())
                    .field("value_selection", &self.get_value_selection())
                    .field("value_unit", &self.get_value_unit())
                    .field("meter_reading_value", &self.get_meter_reading_value())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropMeterStatusError {
            NoFault = 0,
            GeneralError = 1,
            BusUnconfigured = 2,
            BusUnconnected = 3,
            BusShortcut = 4,
            CommunicationTimeout = 5,
            UnknownProtocolOr = 6,
            BusInitializationRunning = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropCommandId {
            Id08 = 8,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropMeterBusType {
            Reserved = 0,
            Mbus = 1,
            S0 = 2,
            D0 = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropValueSelection {
            Meter1CurrentValue = 0,
            Meter1AccumulatedValue = 1,
            Meter2CurrentValue = 2,
            Meter2AccumulatedValue = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropValueUnit {
            W = 0,
            Wh = 1,
            Kwh = 2,
            M3H = 3,
            Dm3H = 4,
            M3 = 5,
            Dm3 = 6,
            _1DigitalCounter = 7,
            _Other(u8),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func31<'b> {
        Type00Case0(func31::Type00Case0<'b>),
        Type00Case1(func31::Type00Case1<'b>),
        Type00Case2(func31::Type00Case2<'b>),
        Type00Case3(func31::Type00Case3<'b>),
        Type00Case4(func31::Type00Case4<'b>),
    }
    impl<'b> Func31<'b> {
        pub fn type00_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 5> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type00Case0(
                func31::Type00Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case1(
                func31::Type00Case1::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case2(
                func31::Type00Case2::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case3(
                func31::Type00Case3::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case4(
                func31::Type00Case4::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 5>> {
            match type_code {
                0x00 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type00_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// A.C. Current Clamp (D2-32)
    pub mod func32 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Type 0x00 (D2-32-00)
        #[derive(Clone, Copy)]
        pub struct Type00<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Power Fail value.
            pub fn get_power_fail_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(0, 1)
            }
            /// Get the Power Fail value.
            pub fn get_power_fail(&self) -> Option<Type00PropPowerFail> {
                let raw_value = self.get_power_fail_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Divisor value.
            pub fn get_divisor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(1, 1)
            }
            /// Get the Divisor value.
            pub fn get_divisor(&self) -> Option<Type00PropDivisor> {
                let raw_value = self.get_divisor_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Channel 1 value.
            pub fn get_channel_1_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(8, 12)
            }
        }
        impl<'b> ::core::fmt::Debug for Type00<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00")
                    .field("power_fail", &self.get_power_fail())
                    .field("divisor", &self.get_divisor())
                    .field("channel_1", &self.get_channel_1_raw())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00PropPowerFail {
            False = false,
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00PropDivisor {
            X1 = false,
            X10 = true,
            _Other(bool),
        }
        /// Type 0x01 (D2-32-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Power Fail value.
            pub fn get_power_fail_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(0, 1)
            }
            /// Get the Power Fail value.
            pub fn get_power_fail(&self) -> Option<Type01PropPowerFail> {
                let raw_value = self.get_power_fail_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Divisor value.
            pub fn get_divisor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(1, 1)
            }
            /// Get the Divisor value.
            pub fn get_divisor(&self) -> Option<Type01PropDivisor> {
                let raw_value = self.get_divisor_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Channel 1 value.
            pub fn get_channel_1_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(8, 12)
            }

            /// Get the raw Channel 2 value.
            pub fn get_channel_2_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(20, 12)
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("power_fail", &self.get_power_fail())
                    .field("divisor", &self.get_divisor())
                    .field("channel_1", &self.get_channel_1_raw())
                    .field("channel_2", &self.get_channel_2_raw())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropPowerFail {
            False = false,
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropDivisor {
            X1 = false,
            X10 = true,
            _Other(bool),
        }
        /// Type 0x02 (D2-32-02)
        #[derive(Clone, Copy)]
        pub struct Type02<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Power Fail value.
            pub fn get_power_fail_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(0, 1)
            }
            /// Get the Power Fail value.
            pub fn get_power_fail(&self) -> Option<Type02PropPowerFail> {
                let raw_value = self.get_power_fail_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Divisor value.
            pub fn get_divisor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(1, 1)
            }
            /// Get the Divisor value.
            pub fn get_divisor(&self) -> Option<Type02PropDivisor> {
                let raw_value = self.get_divisor_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Channel 1 value.
            pub fn get_channel_1_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(8, 12)
            }

            /// Get the raw Channel 2 value.
            pub fn get_channel_2_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(20, 12)
            }

            /// Get the raw Channel 3 value.
            pub fn get_channel_3_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(32, 12)
            }
        }
        impl<'b> ::core::fmt::Debug for Type02<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02")
                    .field("power_fail", &self.get_power_fail())
                    .field("divisor", &self.get_divisor())
                    .field("channel_1", &self.get_channel_1_raw())
                    .field("channel_2", &self.get_channel_2_raw())
                    .field("channel_3", &self.get_channel_3_raw())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropPowerFail {
            False = false,
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropDivisor {
            X1 = false,
            X10 = true,
            _Other(bool),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func32<'b> {
        Type00(func32::Type00<'b>),
        Type01(func32::Type01<'b>),
        Type02(func32::Type02<'b>),
    }
    impl<'b> Func32<'b> {
        pub fn type00_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type00(
                func32::Type00::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func32::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02(
                func32::Type02::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x00 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type00_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Intelligent, Bi-directional Heaters and Controllers (D2-33)
    pub mod func33 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Type 0x00 (D2-33-00), case 0
        #[derive(Clone, Copy)]
        pub struct Type00Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Message Identifier value.
            pub fn get_message_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Message Identifier value.
            pub fn get_message_identifier(&self) -> Option<Type00Case0PropMessageIdentifier> {
                let raw_value = self.get_message_identifier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Request Frame value.
            pub fn get_request_frame_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Request Frame value.
            pub fn get_request_frame(&self) -> Option<Type00Case0PropRequestFrame> {
                let raw_value = self.get_request_frame_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw External Temperature value.
            pub fn get_external_temperature_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(8, 9)
            }
            /// Get the External Temperature value in units of °C.
            pub fn get_external_temperature(&self) -> Option<f64> {
                let raw_value = self.get_external_temperature_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 500.0, 0.1, 50.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case0")
                    .field("message_identifier", &self.get_message_identifier())
                    .field("request_frame", &self.get_request_frame())
                    .field("external_temperature", &self.get_external_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case0PropMessageIdentifier {
            GatewayRequestMessageType = 0,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case0PropRequestFrame {
            QuestionStatusAndFlags = 8,
            QuestionParametersHeaters = 9,
            QuestionSensorCoHygroSound = 10,
            QuestionSensorParticleRadioactivity = 11,
            QuestionSensorAirFlowHygrometryPressureAndTemperature = 12,
            InformationToHeater = 13,
            Reserved = 14,
            AcknowledgeFrame = 15,
            _Other(u8),
        }
        /// Type 0x00 (D2-33-00), case 1
        #[derive(Clone, Copy)]
        pub struct Type00Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Message Identifier value.
            pub fn get_message_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Message Identifier value.
            pub fn get_message_identifier(&self) -> Option<Type00Case1PropMessageIdentifier> {
                let raw_value = self.get_message_identifier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Window Open Detection Status value.
            pub fn get_window_open_detection_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(4, 1)
            }
            /// Get the Window Open Detection Status value.
            pub fn get_window_open_detection_status(&self) -> Option<Type00Case1PropWindowOpenDetectionStatus> {
                let raw_value = self.get_window_open_detection_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw PIR Detection Status value.
            pub fn get_pir_detection_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(5, 1)
            }
            /// Get the PIR Detection Status value.
            pub fn get_pir_detection_status(&self) -> Option<Type00Case1PropPirDetectionStatus> {
                let raw_value = self.get_pir_detection_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Reference Temperature Status value.
            pub fn get_reference_temperature_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(6, 1)
            }
            /// Get the Reference Temperature Status value.
            pub fn get_reference_temperature_status(&self) -> Option<Type00Case1PropReferenceTemperatureStatus> {
                let raw_value = self.get_reference_temperature_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw COV Sensor value.
            pub fn get_cov_sensor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(7, 1)
            }
            /// Get the COV Sensor value.
            pub fn get_cov_sensor(&self) -> Option<Type00Case1PropCovSensor> {
                let raw_value = self.get_cov_sensor_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw CO Sensor value.
            pub fn get_co_sensor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(8, 1)
            }
            /// Get the CO Sensor value.
            pub fn get_co_sensor(&self) -> Option<Type00Case1PropCoSensor> {
                let raw_value = self.get_co_sensor_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw CO2 Sensor value.
            pub fn get_co2_sensor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(9, 1)
            }
            /// Get the CO2 Sensor value.
            pub fn get_co2_sensor(&self) -> Option<Type00Case1PropCo2Sensor> {
                let raw_value = self.get_co2_sensor_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Particles 1 Sensor value.
            pub fn get_particles_1_sensor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(10, 1)
            }
            /// Get the Particles 1 Sensor value.
            pub fn get_particles_1_sensor(&self) -> Option<Type00Case1PropParticles1Sensor> {
                let raw_value = self.get_particles_1_sensor_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Particles 2.5 Sensor value.
            pub fn get_particles_2_5_sensor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(11, 1)
            }
            /// Get the Particles 2.5 Sensor value.
            pub fn get_particles_2_5_sensor(&self) -> Option<Type00Case1PropParticles25Sensor> {
                let raw_value = self.get_particles_2_5_sensor_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Particles 10 Sensor value.
            pub fn get_particles_10_sensor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(12, 1)
            }
            /// Get the Particles 10 Sensor value.
            pub fn get_particles_10_sensor(&self) -> Option<Type00Case1PropParticles10Sensor> {
                let raw_value = self.get_particles_10_sensor_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Radio Activity Sensor value.
            pub fn get_radio_activity_sensor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(13, 1)
            }
            /// Get the Radio Activity Sensor value.
            pub fn get_radio_activity_sensor(&self) -> Option<Type00Case1PropRadioActivitySensor> {
                let raw_value = self.get_radio_activity_sensor_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Sound Sensor value.
            pub fn get_sound_sensor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(14, 1)
            }
            /// Get the Sound Sensor value.
            pub fn get_sound_sensor(&self) -> Option<Type00Case1PropSoundSensor> {
                let raw_value = self.get_sound_sensor_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Hygrometry Sensor value.
            pub fn get_hygrometry_sensor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(15, 1)
            }
            /// Get the Hygrometry Sensor value.
            pub fn get_hygrometry_sensor(&self) -> Option<Type00Case1PropHygrometrySensor> {
                let raw_value = self.get_hygrometry_sensor_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Air Moving Sensor value.
            pub fn get_air_moving_sensor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(16, 1)
            }
            /// Get the Air Moving Sensor value.
            pub fn get_air_moving_sensor(&self) -> Option<Type00Case1PropAirMovingSensor> {
                let raw_value = self.get_air_moving_sensor_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Pressure Sensor value.
            pub fn get_pressure_sensor_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(17, 1)
            }
            /// Get the Pressure Sensor value.
            pub fn get_pressure_sensor(&self) -> Option<Type00Case1PropPressureSensor> {
                let raw_value = self.get_pressure_sensor_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature Scale Status value.
            pub fn get_temperature_scale_status_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(18, 2)
            }
            /// Get the Temperature Scale Status value.
            pub fn get_temperature_scale_status(&self) -> Option<Type00Case1PropTemperatureScaleStatus> {
                let raw_value = self.get_temperature_scale_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Time Notation Status value.
            pub fn get_time_notation_status_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(20, 2)
            }
            /// Get the Time Notation Status value.
            pub fn get_time_notation_status(&self) -> Option<Type00Case1PropTimeNotationStatus> {
                let raw_value = self.get_time_notation_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Display Content Status value.
            pub fn get_display_content_status_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(22, 3)
            }
            /// Get the Display Content Status value.
            pub fn get_display_content_status(&self) -> Option<Type00Case1PropDisplayContentStatus> {
                let raw_value = self.get_display_content_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Derogation Status value.
            pub fn get_derogation_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(25, 1)
            }
            /// Get the Derogation Status value.
            pub fn get_derogation_status(&self) -> Option<Type00Case1PropDerogationStatus> {
                let raw_value = self.get_derogation_status_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case1")
                    .field("message_identifier", &self.get_message_identifier())
                    .field("window_open_detection_status", &self.get_window_open_detection_status())
                    .field("pir_detection_status", &self.get_pir_detection_status())
                    .field("reference_temperature_status", &self.get_reference_temperature_status())
                    .field("cov_sensor", &self.get_cov_sensor())
                    .field("co_sensor", &self.get_co_sensor())
                    .field("co2_sensor", &self.get_co2_sensor())
                    .field("particles_1_sensor", &self.get_particles_1_sensor())
                    .field("particles_2_5_sensor", &self.get_particles_2_5_sensor())
                    .field("particles_10_sensor", &self.get_particles_10_sensor())
                    .field("radio_activity_sensor", &self.get_radio_activity_sensor())
                    .field("sound_sensor", &self.get_sound_sensor())
                    .field("hygrometry_sensor", &self.get_hygrometry_sensor())
                    .field("air_moving_sensor", &self.get_air_moving_sensor())
                    .field("pressure_sensor", &self.get_pressure_sensor())
                    .field("temperature_scale_status", &self.get_temperature_scale_status())
                    .field("time_notation_status", &self.get_time_notation_status())
                    .field("display_content_status", &self.get_display_content_status())
                    .field("derogation_status", &self.get_derogation_status())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropMessageIdentifier {
            SensorParameters = 1,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropWindowOpenDetectionStatus {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropPirDetectionStatus {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropReferenceTemperatureStatus {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropCovSensor {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropCoSensor {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropCo2Sensor {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropParticles1Sensor {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropParticles25Sensor {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropParticles10Sensor {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropRadioActivitySensor {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropSoundSensor {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropHygrometrySensor {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropAirMovingSensor {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropPressureSensor {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropTemperatureScaleStatus {
            NoChange = 0,
            Default = 1,
            DegreesCelsius = 2,
            DegreesFahrenheit = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropTimeNotationStatus {
            NoChange = 0,
            Default = 1,
            _24H = 2,
            _12H = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropDisplayContentStatus {
            NoChange = 0,
            Default = 1,
            Time = 2,
            RoomTemperatureInternal = 3,
            RoomTemperatureExternal = 4,
            TemperatureSetpoint = 5,
            DisplayOff = 6,
            Reserved = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropDerogationStatus {
            DerogationIsNotAllowed = false,
            DerogationIsAllowed = true,
            _Other(bool),
        }
        /// Type 0x00 (D2-33-00), case 2
        #[derive(Clone, Copy)]
        pub struct Type00Case2<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case2<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Message Identifier value.
            pub fn get_message_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Message Identifier value.
            pub fn get_message_identifier(&self) -> Option<Type00Case2PropMessageIdentifier> {
                let raw_value = self.get_message_identifier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Scheduled Order Type value.
            pub fn get_scheduled_order_type_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(4, 1)
            }
            /// Get the Scheduled Order Type value.
            pub fn get_scheduled_order_type(&self) -> Option<Type00Case2PropScheduledOrderType> {
                let raw_value = self.get_scheduled_order_type_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw End Time Day value.
            pub fn get_end_time_day_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(5, 3)
            }
            /// Get the End Time Day value.
            pub fn get_end_time_day(&self) -> Option<Type00Case2PropEndTimeDay> {
                let raw_value = self.get_end_time_day_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw End Time Minute value.
            pub fn get_end_time_minute_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 6)
            }
            /// Get the End Time Minute value in units of Min.
            pub fn get_end_time_minute(&self) -> Option<f64> {
                let raw_value = self.get_end_time_minute_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 59.0, 0.0, 59.0))
            }

            /// Get the raw End Time Hour value.
            pub fn get_end_time_hour_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(14, 5)
            }
            /// Get the End Time Hour value in units of Hour.
            pub fn get_end_time_hour(&self) -> Option<f64> {
                let raw_value = self.get_end_time_hour_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 23.0, 0.0, 23.0))
            }

            /// Get the raw Start Time Day value.
            pub fn get_start_time_day_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(19, 3)
            }
            /// Get the Start Time Day value.
            pub fn get_start_time_day(&self) -> Option<Type00Case2PropStartTimeDay> {
                let raw_value = self.get_start_time_day_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Start Time Minute value.
            pub fn get_start_time_minute_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(22, 6)
            }
            /// Get the Start Time Minute value in units of Min.
            pub fn get_start_time_minute(&self) -> Option<f64> {
                let raw_value = self.get_start_time_minute_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 59.0, 0.0, 59.0))
            }

            /// Get the raw Start Time Hour value.
            pub fn get_start_time_hour_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(28, 5)
            }
            /// Get the Start Time Hour value in units of Hour.
            pub fn get_start_time_hour(&self) -> Option<f64> {
                let raw_value = self.get_start_time_hour_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 23.0, 0.0, 23.0))
            }

            /// Get the raw Temperature Setpoint value.
            pub fn get_temperature_setpoint_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(33, 9)
            }
            /// Get the Temperature Setpoint value in units of °C.
            pub fn get_temperature_setpoint(&self) -> Option<f64> {
                let raw_value = self.get_temperature_setpoint_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 500.0, 0.1, 50.0))
            }

            /// Get the raw Clear Schedule value.
            pub fn get_clear_schedule_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(42, 1)
            }
            /// Get the Clear Schedule value.
            pub fn get_clear_schedule(&self) -> Option<Type00Case2PropClearSchedule> {
                let raw_value = self.get_clear_schedule_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case2<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case2")
                    .field("message_identifier", &self.get_message_identifier())
                    .field("scheduled_order_type", &self.get_scheduled_order_type())
                    .field("end_time_day", &self.get_end_time_day())
                    .field("end_time_minute", &self.get_end_time_minute())
                    .field("end_time_hour", &self.get_end_time_hour())
                    .field("start_time_day", &self.get_start_time_day())
                    .field("start_time_minute", &self.get_start_time_minute())
                    .field("start_time_hour", &self.get_start_time_hour())
                    .field("temperature_setpoint", &self.get_temperature_setpoint())
                    .field("clear_schedule", &self.get_clear_schedule())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropMessageIdentifier {
            Program = 2,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropScheduledOrderType {
            OneTime = false,
            Weekly = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropEndTimeDay {
            Monday = 0,
            Tuesday = 1,
            Wednesday = 2,
            Thursday = 3,
            Friday = 4,
            Saturday = 5,
            Sunday = 6,
            Reserved = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropStartTimeDay {
            Monday = 0,
            Tuesday = 1,
            Wednesday = 2,
            Thursday = 3,
            Friday = 4,
            Saturday = 5,
            Sunday = 6,
            Reserved = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropClearSchedule {
            Set = false,
            Clear = true,
            _Other(bool),
        }
        /// Type 0x00 (D2-33-00), case 3
        #[derive(Clone, Copy)]
        pub struct Type00Case3<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case3<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Message Identifier value.
            pub fn get_message_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Message Identifier value.
            pub fn get_message_identifier(&self) -> Option<Type00Case3PropMessageIdentifier> {
                let raw_value = self.get_message_identifier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Day value.
            pub fn get_day_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 5)
            }
            /// Get the Day value in units of Day.
            pub fn get_day(&self) -> Option<f64> {
                let raw_value = self.get_day_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 31.0, 1.0, 31.0))
            }

            /// Get the raw Month value.
            pub fn get_month_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(9, 4)
            }
            /// Get the Month value in units of Mon.
            pub fn get_month(&self) -> Option<f64> {
                let raw_value = self.get_month_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 12.0, 1.0, 12.0))
            }

            /// Get the raw Year value.
            pub fn get_year_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(13, 12)
            }
            /// Get the Year value in units of Year.
            pub fn get_year(&self) -> Option<f64> {
                let raw_value = self.get_year_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 4095.0, 0.0, 4095.0))
            }

            /// Get the raw Minute value.
            pub fn get_minute_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(25, 6)
            }
            /// Get the Minute value in units of Min.
            pub fn get_minute(&self) -> Option<f64> {
                let raw_value = self.get_minute_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 59.0, 0.0, 59.0))
            }

            /// Get the raw Hour value.
            pub fn get_hour_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(31, 5)
            }
            /// Get the Hour value in units of Hour.
            pub fn get_hour(&self) -> Option<f64> {
                let raw_value = self.get_hour_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 23.0, 0.0, 23.0))
            }

            /// Get the raw Day Week value.
            pub fn get_day_week_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(36, 3)
            }
            /// Get the Day Week value.
            pub fn get_day_week(&self) -> Option<Type00Case3PropDayWeek> {
                let raw_value = self.get_day_week_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case3<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case3")
                    .field("message_identifier", &self.get_message_identifier())
                    .field("day", &self.get_day())
                    .field("month", &self.get_month())
                    .field("year", &self.get_year())
                    .field("minute", &self.get_minute())
                    .field("hour", &self.get_hour())
                    .field("day_week", &self.get_day_week())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case3PropMessageIdentifier {
            TimeAndDate = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case3PropDayWeek {
            Monday = 0,
            Tuesday = 1,
            Wednesday = 2,
            Thursday = 3,
            Friday = 4,
            Saturday = 5,
            Sunday = 6,
            Reserved = 7,
            _Other(u8),
        }
        /// Type 0x00 (D2-33-00), case 4
        #[derive(Clone, Copy)]
        pub struct Type00Case4<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case4<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Message Identifier value.
            pub fn get_message_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Message Identifier value.
            pub fn get_message_identifier(&self) -> Option<Type00Case4PropMessageIdentifier> {
                let raw_value = self.get_message_identifier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Request Frame value.
            pub fn get_request_frame_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Request Frame value.
            pub fn get_request_frame(&self) -> Option<Type00Case4PropRequestFrame> {
                let raw_value = self.get_request_frame_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Error Flag value.
            pub fn get_error_flag_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(8, 16)
            }
            /// Get the Error Flag value.
            pub fn get_error_flag(&self) -> Option<Type00Case4PropErrorFlag> {
                let raw_value = self.get_error_flag_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Heating Flag value.
            pub fn get_heating_flag_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(24, 1)
            }
            /// Get the Heating Flag value.
            pub fn get_heating_flag(&self) -> Option<Type00Case4PropHeatingFlag> {
                let raw_value = self.get_heating_flag_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Pilot Wire Flag value.
            pub fn get_pilot_wire_flag_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(25, 2)
            }
            /// Get the Pilot Wire Flag value.
            pub fn get_pilot_wire_flag(&self) -> Option<Type00Case4PropPilotWireFlag> {
                let raw_value = self.get_pilot_wire_flag_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Window Open Detection Flag value.
            pub fn get_window_open_detection_flag_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(27, 2)
            }
            /// Get the Window Open Detection Flag value.
            pub fn get_window_open_detection_flag(&self) -> Option<Type00Case4PropWindowOpenDetectionFlag> {
                let raw_value = self.get_window_open_detection_flag_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw PIR Flag value.
            pub fn get_pir_flag_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(29, 2)
            }
            /// Get the PIR Flag value.
            pub fn get_pir_flag(&self) -> Option<Type00Case4PropPirFlag> {
                let raw_value = self.get_pir_flag_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Key Lock User Status value.
            pub fn get_key_lock_user_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(31, 1)
            }
            /// Get the Key Lock User Status value.
            pub fn get_key_lock_user_status(&self) -> Option<Type00Case4PropKeyLockUserStatus> {
                let raw_value = self.get_key_lock_user_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Reference Temperature Flag value.
            pub fn get_reference_temperature_flag_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(32, 1)
            }
            /// Get the Reference Temperature Flag value.
            pub fn get_reference_temperature_flag(&self) -> Option<Type00Case4PropReferenceTemperatureFlag> {
                let raw_value = self.get_reference_temperature_flag_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Derogation Flag value.
            pub fn get_derogation_flag_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(33, 1)
            }
            /// Get the Derogation Flag value.
            pub fn get_derogation_flag(&self) -> Option<Type00Case4PropDerogationFlag> {
                let raw_value = self.get_derogation_flag_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Internal Temperature value.
            pub fn get_internal_temperature_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(34, 9)
            }
            /// Get the Internal Temperature value in units of °C.
            pub fn get_internal_temperature(&self) -> Option<f64> {
                let raw_value = self.get_internal_temperature_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 500.0, 0.1, 50.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case4<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case4")
                    .field("message_identifier", &self.get_message_identifier())
                    .field("request_frame", &self.get_request_frame())
                    .field("error_flag", &self.get_error_flag())
                    .field("heating_flag", &self.get_heating_flag())
                    .field("pilot_wire_flag", &self.get_pilot_wire_flag())
                    .field("window_open_detection_flag", &self.get_window_open_detection_flag())
                    .field("pir_flag", &self.get_pir_flag())
                    .field("key_lock_user_status", &self.get_key_lock_user_status())
                    .field("reference_temperature_flag", &self.get_reference_temperature_flag())
                    .field("derogation_flag", &self.get_derogation_flag())
                    .field("internal_temperature", &self.get_internal_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropMessageIdentifier {
            RequestAndStatus = 8,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropRequestFrame {
            QuestionExternalTemp = 0,
            QuestionSensorParameters = 1,
            QuestionProgram = 2,
            QuestionTimeAndDate = 3,
            InformationToGateway = 4,
            AcknowledgeFrame = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u16, derive_compare = "as_int")]
        pub enum Type00Case4PropErrorFlag {
            TemperatureSensorIsOpen = 0,
            TemperatureSensorIsShortCircuit = 1,
            TemperatureMeasuredIsGreaterThan50DegreesC = 2,
            ErrorBetweenInternalTempAndExternalTempIsGreaterThan4DegreesC = 3,
            Reserved = 4,
            Reserved1 = 5,
            Reserved2 = 6,
            Reserved3 = 7,
            Reserved4 = 8,
            Reserved5 = 9,
            Reserved6 = 10,
            Reserved7 = 11,
            Reserved8 = 12,
            Reserved9 = 13,
            Reserved10 = 14,
            Reserved11 = 15,
            _Other(u16),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case4PropHeatingFlag {
            NoHeatingUp = false,
            HeatingUp = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropPilotWireFlag {
            NoPilotWire = 0,
            PilotWireActive = 1,
            PilotWireMinus1 = 2,
            PilotWireMinus2 = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropWindowOpenDetectionFlag {
            Disabled = 0,
            Close = 1,
            Open = 2,
            Reserved = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropPirFlag {
            Disabled = 0,
            NoMovementDetected = 1,
            MovementDetected = 2,
            Reserved = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case4PropKeyLockUserStatus {
            KeyLockIsDisabled = false,
            KeyLockIsEnabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case4PropReferenceTemperatureFlag {
            Internal = false,
            External = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case4PropDerogationFlag {
            NoDerogation = false,
            DerogationActive = true,
            _Other(bool),
        }
        /// Type 0x00 (D2-33-00), case 5
        #[derive(Clone, Copy)]
        pub struct Type00Case5<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case5<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Message Identifier value.
            pub fn get_message_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Message Identifier value.
            pub fn get_message_identifier(&self) -> Option<Type00Case5PropMessageIdentifier> {
                let raw_value = self.get_message_identifier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy Measurement value.
            pub fn get_energy_measurement_raw(&self) -> Option<u32> {
                self.reversed_bytes.u32_from_bits(4, 24)
            }
            /// Get the Energy Measurement value in units of kWh.
            pub fn get_energy_measurement(&self) -> Option<f64> {
                let raw_value = self.get_energy_measurement_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 16777215.0, 0.0, 1677721.0))
            }

            /// Get the raw Derogation Temperature Setpoint value.
            pub fn get_derogation_temperature_setpoint_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(28, 9)
            }
            /// Get the Derogation Temperature Setpoint value in units of °C.
            pub fn get_derogation_temperature_setpoint(&self) -> Option<f64> {
                let raw_value = self.get_derogation_temperature_setpoint_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 500.0, 0.1, 50.0))
            }

            /// Get the raw Firmware Version value.
            pub fn get_firmware_version_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(37, 10)
            }
            /// Get the Firmware Version value in units of -.
            pub fn get_firmware_version(&self) -> Option<f64> {
                let raw_value = self.get_firmware_version_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 1024.0, 0.0, 1024.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case5<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case5")
                    .field("message_identifier", &self.get_message_identifier())
                    .field("energy_measurement", &self.get_energy_measurement())
                    .field("derogation_temperature_setpoint", &self.get_derogation_temperature_setpoint())
                    .field("firmware_version", &self.get_firmware_version())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case5PropMessageIdentifier {
            HeaterParameters = 9,
            _Other(u8),
        }
        /// Type 0x00 (D2-33-00), case 6
        #[derive(Clone, Copy)]
        pub struct Type00Case6<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case6<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Message Identifier value.
            pub fn get_message_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Message Identifier value.
            pub fn get_message_identifier(&self) -> Option<Type00Case6PropMessageIdentifier> {
                let raw_value = self.get_message_identifier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw COV Value value.
            pub fn get_cov_value_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(4, 16)
            }
            /// Get the COV Value value in units of ppb.
            pub fn get_cov_value(&self) -> Option<f64> {
                let raw_value = self.get_cov_value_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 65535.0, 1.0, 65535.0))
            }

            /// Get the raw CO Value value.
            pub fn get_co_value_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(20, 8)
            }
            /// Get the CO Value value in units of ppm.
            pub fn get_co_value(&self) -> Option<f64> {
                let raw_value = self.get_co_value_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 255.0, 1.0, 255.0))
            }

            /// Get the raw CO2 Value value.
            pub fn get_co2_value_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(28, 8)
            }
            /// Get the CO2 Value value in units of ppm.
            pub fn get_co2_value(&self) -> Option<f64> {
                let raw_value = self.get_co2_value_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 255.0, 10.0, 2550.0))
            }

            /// Get the raw Sound Value value.
            pub fn get_sound_value_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(36, 7)
            }
            /// Get the Sound Value value in units of dB.
            pub fn get_sound_value(&self) -> Option<f64> {
                let raw_value = self.get_sound_value_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 127.0, 1.0, 127.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case6<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case6")
                    .field("message_identifier", &self.get_message_identifier())
                    .field("cov_value", &self.get_cov_value())
                    .field("co_value", &self.get_co_value())
                    .field("co2_value", &self.get_co2_value())
                    .field("sound_value", &self.get_sound_value())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case6PropMessageIdentifier {
            ValueOfCoCovCo2AndSoundLevel = 10,
            _Other(u8),
        }
        /// Type 0x00 (D2-33-00), case 7
        #[derive(Clone, Copy)]
        pub struct Type00Case7<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case7<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Message Identifier value.
            pub fn get_message_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Message Identifier value.
            pub fn get_message_identifier(&self) -> Option<Type00Case7PropMessageIdentifier> {
                let raw_value = self.get_message_identifier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Particle 1 Value value.
            pub fn get_particle_1_value_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(4, 9)
            }
            /// Get the Particle 1 Value value in units of μg/m3.
            pub fn get_particle_1_value(&self) -> Option<f64> {
                let raw_value = self.get_particle_1_value_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 511.0, 1.0, 511.0))
            }

            /// Get the raw Particle 2 Value value.
            pub fn get_particle_2_value_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(13, 9)
            }
            /// Get the Particle 2 Value value in units of μg/m3.
            pub fn get_particle_2_value(&self) -> Option<f64> {
                let raw_value = self.get_particle_2_value_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 511.0, 1.0, 511.0))
            }

            /// Get the raw Particle 10 Value value.
            pub fn get_particle_10_value_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(22, 9)
            }
            /// Get the Particle 10 Value value in units of μg/m3.
            pub fn get_particle_10_value(&self) -> Option<f64> {
                let raw_value = self.get_particle_10_value_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 511.0, 1.0, 511.0))
            }

            /// Get the raw Radioactivity Value value.
            pub fn get_radioactivity_value_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(31, 14)
            }
            /// Get the Radioactivity Value value in units of μSv/h.
            pub fn get_radioactivity_value(&self) -> Option<f64> {
                let raw_value = self.get_radioactivity_value_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 16383.0, 0.01, 163.83))
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case7<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case7")
                    .field("message_identifier", &self.get_message_identifier())
                    .field("particle_1_value", &self.get_particle_1_value())
                    .field("particle_2_value", &self.get_particle_2_value())
                    .field("particle_10_value", &self.get_particle_10_value())
                    .field("radioactivity_value", &self.get_radioactivity_value())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case7PropMessageIdentifier {
            ValueOfParticlesAndRadioactivitySensors = 11,
            _Other(u8),
        }
        /// Type 0x00 (D2-33-00), case 8
        #[derive(Clone, Copy)]
        pub struct Type00Case8<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case8<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Message Identifier value.
            pub fn get_message_identifier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Message Identifier value.
            pub fn get_message_identifier(&self) -> Option<Type00Case8PropMessageIdentifier> {
                let raw_value = self.get_message_identifier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Air Moving value.
            pub fn get_air_moving_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Air Moving value in units of m/s.
            pub fn get_air_moving(&self) -> Option<f64> {
                let raw_value = self.get_air_moving_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 15.0, 1.0, 15.0))
            }

            /// Get the raw Pressure Value value.
            pub fn get_pressure_value_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(15, 10)
            }
            /// Get the Pressure Value value in units of hPa.
            pub fn get_pressure_value(&self) -> Option<f64> {
                let raw_value = self.get_pressure_value_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 1023.0, 500.0, 1150.0))
            }

            /// Get the raw Hygrometry Value value.
            pub fn get_hygrometry_value_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(25, 8)
            }
            /// Get the Hygrometry Value value in units of %.
            pub fn get_hygrometry_value(&self) -> Option<f64> {
                let raw_value = self.get_hygrometry_value_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 200.0, 1.0, 100.0))
            }

            /// Get the raw Internal Temperature value.
            pub fn get_internal_temperature_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(33, 11)
            }
            /// Get the Internal Temperature value in units of °C.
            pub fn get_internal_temperature(&self) -> Option<f64> {
                let raw_value = self.get_internal_temperature_raw()? as f64;
                Some(range_scale(raw_value, 1.0, 500.0, 0.1, 50.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case8<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case8")
                    .field("message_identifier", &self.get_message_identifier())
                    .field("air_moving", &self.get_air_moving())
                    .field("pressure_value", &self.get_pressure_value())
                    .field("hygrometry_value", &self.get_hygrometry_value())
                    .field("internal_temperature", &self.get_internal_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case8PropMessageIdentifier {
            ValueOfAirHygroPressureAndTemp = 12,
            _Other(u8),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func33<'b> {
        Type00Case0(func33::Type00Case0<'b>),
        Type00Case1(func33::Type00Case1<'b>),
        Type00Case2(func33::Type00Case2<'b>),
        Type00Case3(func33::Type00Case3<'b>),
        Type00Case4(func33::Type00Case4<'b>),
        Type00Case5(func33::Type00Case5<'b>),
        Type00Case6(func33::Type00Case6<'b>),
        Type00Case7(func33::Type00Case7<'b>),
        Type00Case8(func33::Type00Case8<'b>),
    }
    impl<'b> Func33<'b> {
        pub fn type00_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 9> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type00Case0(
                func33::Type00Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case1(
                func33::Type00Case1::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case2(
                func33::Type00Case2::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case3(
                func33::Type00Case3::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case4(
                func33::Type00Case4::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case5(
                func33::Type00Case5::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case6(
                func33::Type00Case6::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case7(
                func33::Type00Case7::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case8(
                func33::Type00Case8::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 9>> {
            match type_code {
                0x00 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type00_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Heating Actuator (D2-34)
    pub mod func34 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// 1 Output Channel (D2-34-00), case 0
        #[derive(Clone, Copy)]
        pub struct Type00Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Channel value.
            pub fn get_channel_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 5)
            }
            /// Get the Channel value.
            pub fn get_channel(&self) -> Option<Type00Case0PropChannel> {
                let raw_value = self.get_channel_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(12, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case0PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case0")
                    .field("channel", &self.get_channel())
                    .field("command_id", &self.get_command_id())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case0PropChannel {
            AllChannelsSupportedByTheDevice = 30,
            NotUsed = 31,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case0PropCommandId {
            StatusQueryCommand = 3,
            _Other(u8),
        }
        /// 1 Output Channel (D2-34-00), case 1
        #[derive(Clone, Copy)]
        pub struct Type00Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(0, 9)
            }
            /// Get the Temperature value.
            pub fn get_temperature(&self) -> Option<Type00Case1PropTemperature> {
                let raw_value = self.get_temperature_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Set point value.
            pub fn get_set_point_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(9, 9)
            }
            /// Get the Set point value in units of °C.
            pub fn get_set_point(&self) -> Option<f64> {
                let raw_value = self.get_set_point_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 400.0, 0.0, 40.0))
            }

            /// Get the raw Operation Mode value.
            pub fn get_operation_mode_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(18, 4)
            }
            /// Get the Operation Mode value.
            pub fn get_operation_mode(&self) -> Option<Type00Case1PropOperationMode> {
                let raw_value = self.get_operation_mode_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Channel value.
            pub fn get_channel_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(22, 5)
            }
            /// Get the Channel value.
            pub fn get_channel(&self) -> Option<Type00Case1PropChannel> {
                let raw_value = self.get_channel_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(28, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case1PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case1")
                    .field("temperature", &self.get_temperature())
                    .field("set_point", &self.get_set_point())
                    .field("operation_mode", &self.get_operation_mode())
                    .field("channel", &self.get_channel())
                    .field("command_id", &self.get_command_id())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u16, derive_compare = "as_int")]
        pub enum Type00Case1PropTemperature {
            Unknown = 511,
            _Other(u16),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropOperationMode {
            OffDeactivated = 0,
            TemperatureUnknown = 1,
            NoHeatingRequired = 2,
            HeatingRequired = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropChannel {
            AllChannelsSupportedByTheDevice = 30,
            NotUsed = 31,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropCommandId {
            StatusResponseCommand = 4,
            _Other(u8),
        }
        /// 1 Output Channel (D2-34-00), case 2
        #[derive(Clone, Copy)]
        pub struct Type00Case2<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case2<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Configuration value.
            pub fn get_configuration_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 2)
            }
            /// Get the Configuration value.
            pub fn get_configuration(&self) -> Option<Type00Case2PropConfiguration> {
                let raw_value = self.get_configuration_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Duration value.
            pub fn get_duration_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(2, 6)
            }
            /// Get the Duration value.
            pub fn get_duration(&self) -> Option<Type00Case2PropDuration> {
                let raw_value = self.get_duration_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Set point value.
            pub fn get_set_point_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 7)
            }
            /// Get the Set point value in units of K.
            pub fn get_set_point(&self) -> Option<f64> {
                let raw_value = self.get_set_point_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 100.0, 0.0, 10.0))
            }

            /// Get the raw Set point 1 value.
            pub fn get_set_point_1_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(15, 9)
            }
            /// Get the Set point 1 value in units of °C.
            pub fn get_set_point_1(&self) -> Option<f64> {
                let raw_value = self.get_set_point_1_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 400.0, 0.0, 40.0))
            }

            /// Get the raw Channel value.
            pub fn get_channel_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 5)
            }
            /// Get the Channel value.
            pub fn get_channel(&self) -> Option<Type00Case2PropChannel> {
                let raw_value = self.get_channel_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(36, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case2PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case2<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case2")
                    .field("configuration", &self.get_configuration())
                    .field("duration", &self.get_duration())
                    .field("set_point", &self.get_set_point())
                    .field("set_point_1", &self.get_set_point_1())
                    .field("channel", &self.get_channel())
                    .field("command_id", &self.get_command_id())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropConfiguration {
            RoomPanelValue = 0,
            OverrideWithOvr = 1,
            AddShfToPanelValue = 2,
            SubtractShfFromPanelValue = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropDuration {
            Endless = 0,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropChannel {
            AllChannelsSupportedByTheDevice = 30,
            NotUsed = 31,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropCommandId {
            SetPointConfigurationCommand = 5,
            _Other(u8),
        }
        /// 1 Output Channel (D2-34-00), case 3
        #[derive(Clone, Copy)]
        pub struct Type00Case3<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case3<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Channel value.
            pub fn get_channel_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 5)
            }
            /// Get the Channel value.
            pub fn get_channel(&self) -> Option<Type00Case3PropChannel> {
                let raw_value = self.get_channel_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(12, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case3PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case3<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case3")
                    .field("channel", &self.get_channel())
                    .field("command_id", &self.get_command_id())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case3PropChannel {
            AllChannelsSupportedByTheDevice = 30,
            NotUsed = 31,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case3PropCommandId {
            SetPointQueryCommand = 6,
            _Other(u8),
        }
        /// 1 Output Channel (D2-34-00), case 4
        #[derive(Clone, Copy)]
        pub struct Type00Case4<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case4<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Configuration value.
            pub fn get_configuration_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 2)
            }
            /// Get the Configuration value.
            pub fn get_configuration(&self) -> Option<Type00Case4PropConfiguration> {
                let raw_value = self.get_configuration_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Duration value.
            pub fn get_duration_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(2, 6)
            }
            /// Get the Duration value.
            pub fn get_duration(&self) -> Option<Type00Case4PropDuration> {
                let raw_value = self.get_duration_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Set point value.
            pub fn get_set_point_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(8, 9)
            }
            /// Get the Set point value in units of °C.
            pub fn get_set_point(&self) -> Option<f64> {
                let raw_value = self.get_set_point_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 400.0, 0.0, 40.0))
            }

            /// Get the raw Set point 1 value.
            pub fn get_set_point_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(17, 7)
            }
            /// Get the Set point 1 value in units of K.
            pub fn get_set_point_1(&self) -> Option<f64> {
                let raw_value = self.get_set_point_1_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 100.0, 0.0, 10.0))
            }

            /// Get the raw Set point 2 value.
            pub fn get_set_point_2_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(24, 9)
            }
            /// Get the Set point 2 value in units of °C.
            pub fn get_set_point_2(&self) -> Option<f64> {
                let raw_value = self.get_set_point_2_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 400.0, 0.0, 40.0))
            }

            /// Get the raw Channel value.
            pub fn get_channel_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(33, 5)
            }
            /// Get the Channel value.
            pub fn get_channel(&self) -> Option<Type00Case4PropChannel> {
                let raw_value = self.get_channel_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(44, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case4PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case4<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case4")
                    .field("configuration", &self.get_configuration())
                    .field("duration", &self.get_duration())
                    .field("set_point", &self.get_set_point())
                    .field("set_point_1", &self.get_set_point_1())
                    .field("set_point_2", &self.get_set_point_2())
                    .field("channel", &self.get_channel())
                    .field("command_id", &self.get_command_id())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropConfiguration {
            RoomPanelValue = 0,
            OverrideWithOvr = 1,
            AddShfToPanelValue = 2,
            SubtractShfFromPanelValue = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropDuration {
            ExpiredCfg0 = 0,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropChannel {
            AllChannelsSupportedByTheDevice = 30,
            NotUsed = 31,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropCommandId {
            SetPointResponseCommand = 7,
            _Other(u8),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func34<'b> {
        Type00Case0(func34::Type00Case0<'b>),
        Type00Case1(func34::Type00Case1<'b>),
        Type00Case2(func34::Type00Case2<'b>),
        Type00Case3(func34::Type00Case3<'b>),
        Type00Case4(func34::Type00Case4<'b>),
    }
    impl<'b> Func34<'b> {
        pub fn type00_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 5> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type00Case0(
                func34::Type00Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case1(
                func34::Type00Case1::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case2(
                func34::Type00Case2::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case3(
                func34::Type00Case3::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case4(
                func34::Type00Case4::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 5>> {
            match type_code {
                0x00 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type00_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// LED Controller Status (D2-40)
    pub mod func40 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Type 0x00 (D2-40-00)
        #[derive(Clone, Copy)]
        pub struct Type00<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LED output enabled value.
            pub fn get_led_output_enabled_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(0, 1)
            }
            /// Get the LED output enabled value.
            pub fn get_led_output_enabled(&self) -> Option<Type00PropLedOutputEnabled> {
                let raw_value = self.get_led_output_enabled_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw “Demand Response” mode Active value.
            pub fn get_demand_response_mode_active_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(1, 1)
            }
            /// Get the “Demand Response” mode Active value.
            pub fn get_demand_response_mode_active(&self) -> Option<Type00PropDemandResponseModeActive> {
                let raw_value = self.get_demand_response_mode_active_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Daylight Harvesting Active value.
            pub fn get_daylight_harvesting_active_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(2, 1)
            }
            /// Get the Daylight Harvesting Active value.
            pub fn get_daylight_harvesting_active(&self) -> Option<Type00PropDaylightHarvestingActive> {
                let raw_value = self.get_daylight_harvesting_active_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Occupancy State value.
            pub fn get_occupancy_state_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(3, 2)
            }
            /// Get the Occupancy State value.
            pub fn get_occupancy_state(&self) -> Option<Type00PropOccupancyState> {
                let raw_value = self.get_occupancy_state_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Status Tx reason value.
            pub fn get_status_tx_reason_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(5, 1)
            }
            /// Get the Status Tx reason value.
            pub fn get_status_tx_reason(&self) -> Option<Type00PropStatusTxReason> {
                let raw_value = self.get_status_tx_reason_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw MsgId value.
            pub fn get_msgid_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(6, 2)
            }
            /// Get the MsgId value.
            pub fn get_msgid(&self) -> Option<Type00PropMsgid> {
                let raw_value = self.get_msgid_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Current Dim Level value.
            pub fn get_current_dim_level_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Current Dim Level value.
            pub fn get_current_dim_level(&self) -> Option<Type00PropCurrentDimLevel> {
                let raw_value = self.get_current_dim_level_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00")
                    .field("led_output_enabled", &self.get_led_output_enabled())
                    .field("demand_response_mode_active", &self.get_demand_response_mode_active())
                    .field("daylight_harvesting_active", &self.get_daylight_harvesting_active())
                    .field("occupancy_state", &self.get_occupancy_state())
                    .field("status_tx_reason", &self.get_status_tx_reason())
                    .field("msgid", &self.get_msgid())
                    .field("current_dim_level", &self.get_current_dim_level())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00PropLedOutputEnabled {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00PropDemandResponseModeActive {
            False = false,
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00PropDaylightHarvestingActive {
            False = false,
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropOccupancyState {
            NotOccupied = 0,
            Occupied = 1,
            Unknown = 2,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00PropStatusTxReason {
            Other = false,
            Heartbeat = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropMsgid {
            LedStatusMonocolor = 0,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropCurrentDimLevel {
            IfNotUsed = 255,
            _Other(u8),
        }
        /// Type 0x01 (D2-40-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw LED output enabled value.
            pub fn get_led_output_enabled_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(0, 1)
            }
            /// Get the LED output enabled value.
            pub fn get_led_output_enabled(&self) -> Option<Type01PropLedOutputEnabled> {
                let raw_value = self.get_led_output_enabled_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw “Demand Response” mode Active value.
            pub fn get_demand_response_mode_active_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(1, 1)
            }
            /// Get the “Demand Response” mode Active value.
            pub fn get_demand_response_mode_active(&self) -> Option<Type01PropDemandResponseModeActive> {
                let raw_value = self.get_demand_response_mode_active_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Daylight Harvesting Active value.
            pub fn get_daylight_harvesting_active_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(2, 1)
            }
            /// Get the Daylight Harvesting Active value.
            pub fn get_daylight_harvesting_active(&self) -> Option<Type01PropDaylightHarvestingActive> {
                let raw_value = self.get_daylight_harvesting_active_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Occupancy State value.
            pub fn get_occupancy_state_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(3, 2)
            }
            /// Get the Occupancy State value.
            pub fn get_occupancy_state(&self) -> Option<Type01PropOccupancyState> {
                let raw_value = self.get_occupancy_state_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Status Tx reason value.
            pub fn get_status_tx_reason_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(5, 1)
            }
            /// Get the Status Tx reason value.
            pub fn get_status_tx_reason(&self) -> Option<Type01PropStatusTxReason> {
                let raw_value = self.get_status_tx_reason_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw MsgId value.
            pub fn get_msgid_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(6, 2)
            }
            /// Get the MsgId value.
            pub fn get_msgid(&self) -> Option<Type01PropMsgid> {
                let raw_value = self.get_msgid_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Current Dim Level LED R value.
            pub fn get_current_dim_level_led_r_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Current Dim Level LED R value.
            pub fn get_current_dim_level_led_r(&self) -> Option<Type01PropCurrentDimLevelLedR> {
                let raw_value = self.get_current_dim_level_led_r_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Current Dim Level LED G value.
            pub fn get_current_dim_level_led_g_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Current Dim Level LED G value.
            pub fn get_current_dim_level_led_g(&self) -> Option<Type01PropCurrentDimLevelLedG> {
                let raw_value = self.get_current_dim_level_led_g_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Current Dim Level LED B value.
            pub fn get_current_dim_level_led_b_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 8)
            }
            /// Get the Current Dim Level LED B value.
            pub fn get_current_dim_level_led_b(&self) -> Option<Type01PropCurrentDimLevelLedB> {
                let raw_value = self.get_current_dim_level_led_b_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("led_output_enabled", &self.get_led_output_enabled())
                    .field("demand_response_mode_active", &self.get_demand_response_mode_active())
                    .field("daylight_harvesting_active", &self.get_daylight_harvesting_active())
                    .field("occupancy_state", &self.get_occupancy_state())
                    .field("status_tx_reason", &self.get_status_tx_reason())
                    .field("msgid", &self.get_msgid())
                    .field("current_dim_level_led_r", &self.get_current_dim_level_led_r())
                    .field("current_dim_level_led_g", &self.get_current_dim_level_led_g())
                    .field("current_dim_level_led_b", &self.get_current_dim_level_led_b())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropLedOutputEnabled {
            Disabled = false,
            Enabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropDemandResponseModeActive {
            False = false,
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropDaylightHarvestingActive {
            False = false,
            True = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropOccupancyState {
            NotOccupied = 0,
            Occupied = 1,
            Unknown = 2,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropStatusTxReason {
            Other = false,
            Heartbeat = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropMsgid {
            LedStatusRgb = 1,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropCurrentDimLevelLedR {
            IfNotUsed = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropCurrentDimLevelLedG {
            IfNotUsed = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropCurrentDimLevelLedB {
            IfNotUsed = 255,
            _Other(u8),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func40<'b> {
        Type00(func40::Type00<'b>),
        Type01(func40::Type01<'b>),
    }
    impl<'b> Func40<'b> {
        pub fn type00_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type00(
                func40::Type00::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func40::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x00 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type00_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Status Data, Sensor Data, Maintenance Data, Light Control (D2-41)
    pub mod func41 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Products with Multisensor and/or Light Units (D2-41-00), case 0
        #[derive(Clone, Copy)]
        pub struct Type00Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Unit Index value.
            pub fn get_unit_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Unit Index value.
            pub fn get_unit_index(&self) -> Option<Type00Case0PropUnitIndex> {
                let raw_value = self.get_unit_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case0PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case0")
                    .field("unit_index", &self.get_unit_index())
                    .field("command_id", &self.get_command_id())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case0PropUnitIndex {
            Reserved = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case0PropCommandId {
            GetProductStatus = 0,
            GetUnitStatus = 1,
            GetPresenceData = 2,
            GetEnvironmentalData = 3,
            GetMaintenanceData = 4,
            _Other(u8),
        }
        /// Products with Multisensor and/or Light Units (D2-41-00), case 1
        #[derive(Clone, Copy)]
        pub struct Type00Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case1PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit Activity Status 14 value.
            pub fn get_unit_activity_status_14_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(9, 1)
            }
            /// Get the Unit Activity Status 14 value.
            pub fn get_unit_activity_status_14(&self) -> Option<Type00Case1PropUnitActivityStatus14> {
                let raw_value = self.get_unit_activity_status_14_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit Activity Status 13 value.
            pub fn get_unit_activity_status_13_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(10, 1)
            }
            /// Get the Unit Activity Status 13 value.
            pub fn get_unit_activity_status_13(&self) -> Option<Type00Case1PropUnitActivityStatus13> {
                let raw_value = self.get_unit_activity_status_13_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit Activity Status 12 value.
            pub fn get_unit_activity_status_12_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(11, 1)
            }
            /// Get the Unit Activity Status 12 value.
            pub fn get_unit_activity_status_12(&self) -> Option<Type00Case1PropUnitActivityStatus12> {
                let raw_value = self.get_unit_activity_status_12_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit Activity Status 11 value.
            pub fn get_unit_activity_status_11_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(12, 1)
            }
            /// Get the Unit Activity Status 11 value.
            pub fn get_unit_activity_status_11(&self) -> Option<Type00Case1PropUnitActivityStatus11> {
                let raw_value = self.get_unit_activity_status_11_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit Activity Status 10 value.
            pub fn get_unit_activity_status_10_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(13, 1)
            }
            /// Get the Unit Activity Status 10 value.
            pub fn get_unit_activity_status_10(&self) -> Option<Type00Case1PropUnitActivityStatus10> {
                let raw_value = self.get_unit_activity_status_10_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit Activity Status 9 value.
            pub fn get_unit_activity_status_9_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(14, 1)
            }
            /// Get the Unit Activity Status 9 value.
            pub fn get_unit_activity_status_9(&self) -> Option<Type00Case1PropUnitActivityStatus9> {
                let raw_value = self.get_unit_activity_status_9_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit Activity Status 8 value.
            pub fn get_unit_activity_status_8_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(15, 1)
            }
            /// Get the Unit Activity Status 8 value.
            pub fn get_unit_activity_status_8(&self) -> Option<Type00Case1PropUnitActivityStatus8> {
                let raw_value = self.get_unit_activity_status_8_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit Activity Status 7 value.
            pub fn get_unit_activity_status_7_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(16, 1)
            }
            /// Get the Unit Activity Status 7 value.
            pub fn get_unit_activity_status_7(&self) -> Option<Type00Case1PropUnitActivityStatus7> {
                let raw_value = self.get_unit_activity_status_7_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit Activity Status 6 value.
            pub fn get_unit_activity_status_6_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(17, 1)
            }
            /// Get the Unit Activity Status 6 value.
            pub fn get_unit_activity_status_6(&self) -> Option<Type00Case1PropUnitActivityStatus6> {
                let raw_value = self.get_unit_activity_status_6_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit Activity Status 5 value.
            pub fn get_unit_activity_status_5_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(18, 1)
            }
            /// Get the Unit Activity Status 5 value.
            pub fn get_unit_activity_status_5(&self) -> Option<Type00Case1PropUnitActivityStatus5> {
                let raw_value = self.get_unit_activity_status_5_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit Activity Status 4 value.
            pub fn get_unit_activity_status_4_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(19, 1)
            }
            /// Get the Unit Activity Status 4 value.
            pub fn get_unit_activity_status_4(&self) -> Option<Type00Case1PropUnitActivityStatus4> {
                let raw_value = self.get_unit_activity_status_4_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit Activity Status 3 value.
            pub fn get_unit_activity_status_3_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(20, 1)
            }
            /// Get the Unit Activity Status 3 value.
            pub fn get_unit_activity_status_3(&self) -> Option<Type00Case1PropUnitActivityStatus3> {
                let raw_value = self.get_unit_activity_status_3_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit Activity Status 2 value.
            pub fn get_unit_activity_status_2_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(21, 1)
            }
            /// Get the Unit Activity Status 2 value.
            pub fn get_unit_activity_status_2(&self) -> Option<Type00Case1PropUnitActivityStatus2> {
                let raw_value = self.get_unit_activity_status_2_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit Activity Status 1 value.
            pub fn get_unit_activity_status_1_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(22, 1)
            }
            /// Get the Unit Activity Status 1 value.
            pub fn get_unit_activity_status_1(&self) -> Option<Type00Case1PropUnitActivityStatus1> {
                let raw_value = self.get_unit_activity_status_1_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Unit Activity Status 0 value.
            pub fn get_unit_activity_status_0_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(23, 1)
            }
            /// Get the Unit Activity Status 0 value.
            pub fn get_unit_activity_status_0(&self) -> Option<Type00Case1PropUnitActivityStatus0> {
                let raw_value = self.get_unit_activity_status_0_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case1")
                    .field("command_id", &self.get_command_id())
                    .field("unit_activity_status_14", &self.get_unit_activity_status_14())
                    .field("unit_activity_status_13", &self.get_unit_activity_status_13())
                    .field("unit_activity_status_12", &self.get_unit_activity_status_12())
                    .field("unit_activity_status_11", &self.get_unit_activity_status_11())
                    .field("unit_activity_status_10", &self.get_unit_activity_status_10())
                    .field("unit_activity_status_9", &self.get_unit_activity_status_9())
                    .field("unit_activity_status_8", &self.get_unit_activity_status_8())
                    .field("unit_activity_status_7", &self.get_unit_activity_status_7())
                    .field("unit_activity_status_6", &self.get_unit_activity_status_6())
                    .field("unit_activity_status_5", &self.get_unit_activity_status_5())
                    .field("unit_activity_status_4", &self.get_unit_activity_status_4())
                    .field("unit_activity_status_3", &self.get_unit_activity_status_3())
                    .field("unit_activity_status_2", &self.get_unit_activity_status_2())
                    .field("unit_activity_status_1", &self.get_unit_activity_status_1())
                    .field("unit_activity_status_0", &self.get_unit_activity_status_0())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropCommandId {
            ProductStatus = 5,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropUnitActivityStatus14 {
            NotActive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropUnitActivityStatus13 {
            NotActive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropUnitActivityStatus12 {
            NotActive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropUnitActivityStatus11 {
            NotActive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropUnitActivityStatus10 {
            NotActive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropUnitActivityStatus9 {
            NotActive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropUnitActivityStatus8 {
            NotActive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropUnitActivityStatus7 {
            NotActive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropUnitActivityStatus6 {
            NotActive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropUnitActivityStatus5 {
            NotActive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropUnitActivityStatus4 {
            NotActive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropUnitActivityStatus3 {
            NotActive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropUnitActivityStatus2 {
            NotActive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropUnitActivityStatus1 {
            NotActive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropUnitActivityStatus0 {
            NotActive = false,
            Active = true,
            _Other(bool),
        }
        /// Products with Multisensor and/or Light Units (D2-41-00), case 2
        #[derive(Clone, Copy)]
        pub struct Type00Case2<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case2<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Unit Index value.
            pub fn get_unit_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Unit Index value.
            pub fn get_unit_index(&self) -> Option<Type00Case2PropUnitIndex> {
                let raw_value = self.get_unit_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case2PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Trigger Operating State value.
            pub fn get_trigger_operating_state_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(12, 4)
            }
            /// Get the Trigger Operating State value.
            pub fn get_trigger_operating_state(&self) -> Option<Type00Case2PropTriggerOperatingState> {
                let raw_value = self.get_trigger_operating_state_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case2<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case2")
                    .field("unit_index", &self.get_unit_index())
                    .field("command_id", &self.get_command_id())
                    .field("trigger_operating_state", &self.get_trigger_operating_state())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropUnitIndex {
            Reserved = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropCommandId {
            TriggerUnitOperatingState = 6,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropTriggerOperatingState {
            SetTriggerLightOff = 0,
            SetTriggerReducedLight = 1,
            SetTriggerWorkingLight = 2,
            SetTriggerServiceLight = 3,
            ClearTriggerLightOff = 4,
            ClearTriggerReducedLight = 5,
            ClearTriggerWorkingLight = 6,
            ClearTriggerServiceLight = 7,
            NoChange = 15,
            _Other(u8),
        }
        /// Products with Multisensor and/or Light Units (D2-41-00), case 3
        #[derive(Clone, Copy)]
        pub struct Type00Case3<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case3<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Unit Index value.
            pub fn get_unit_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Unit Index value.
            pub fn get_unit_index(&self) -> Option<Type00Case3PropUnitIndex> {
                let raw_value = self.get_unit_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case3PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Operating State value.
            pub fn get_operating_state_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(12, 4)
            }
            /// Get the Operating State value.
            pub fn get_operating_state(&self) -> Option<Type00Case3PropOperatingState> {
                let raw_value = self.get_operating_state_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case3<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case3")
                    .field("unit_index", &self.get_unit_index())
                    .field("command_id", &self.get_command_id())
                    .field("operating_state", &self.get_operating_state())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case3PropUnitIndex {
            Reserved = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case3PropCommandId {
            SwitchUnitOperatingState = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case3PropOperatingState {
            Off = 0,
            ReducedLight = 1,
            WorkingLight = 2,
            ServiceLight = 3,
            NotSupported = 15,
            _Other(u8),
        }
        /// Products with Multisensor and/or Light Units (D2-41-00), case 4
        #[derive(Clone, Copy)]
        pub struct Type00Case4<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case4<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Unit Index value.
            pub fn get_unit_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Unit Index value.
            pub fn get_unit_index(&self) -> Option<Type00Case4PropUnitIndex> {
                let raw_value = self.get_unit_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case4PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Operating State value.
            pub fn get_operating_state_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 4)
            }
            /// Get the Operating State value.
            pub fn get_operating_state(&self) -> Option<Type00Case4PropOperatingState> {
                let raw_value = self.get_operating_state_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Switch control value.
            pub fn get_switch_control_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(12, 4)
            }
            /// Get the Switch control value.
            pub fn get_switch_control(&self) -> Option<Type00Case4PropSwitchControl> {
                let raw_value = self.get_switch_control_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw VTL value.
            pub fn get_vtl_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 4)
            }
            /// Get the VTL value.
            pub fn get_vtl(&self) -> Option<Type00Case4PropVtl> {
                let raw_value = self.get_vtl_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Fading Mode value.
            pub fn get_fading_mode_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(20, 4)
            }
            /// Get the Fading Mode value.
            pub fn get_fading_mode(&self) -> Option<Type00Case4PropFadingMode> {
                let raw_value = self.get_fading_mode_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Dimming Level value.
            pub fn get_dimming_level_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 8)
            }
            /// Get the Dimming Level value.
            pub fn get_dimming_level(&self) -> Option<Type00Case4PropDimmingLevel> {
                let raw_value = self.get_dimming_level_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Colour Temp value.
            pub fn get_colour_temp_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(34, 14)
            }
            /// Get the Colour Temp value.
            pub fn get_colour_temp(&self) -> Option<Type00Case4PropColourTemp> {
                let raw_value = self.get_colour_temp_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case4<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case4")
                    .field("unit_index", &self.get_unit_index())
                    .field("command_id", &self.get_command_id())
                    .field("operating_state", &self.get_operating_state())
                    .field("switch_control", &self.get_switch_control())
                    .field("vtl", &self.get_vtl())
                    .field("fading_mode", &self.get_fading_mode())
                    .field("dimming_level", &self.get_dimming_level())
                    .field("colour_temp", &self.get_colour_temp())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropUnitIndex {
            Reserved = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropCommandId {
            SetUnitData = 8,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropOperatingState {
            Off = 0,
            ReducedLight = 1,
            WorkingLight = 2,
            ServiceLight = 3,
            NoChange = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropSwitchControl {
            Store = 0,
            Switch = 1,
            NotUsed = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropVtl {
            Off = 0,
            Normal = 1,
            Owl = 2,
            Lark = 3,
            NoChange = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropFadingMode {
            DirectMode = 0,
            RuntimeMode = 1,
            NotUsed = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case4PropDimmingLevel {
            NoChange = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u16, derive_compare = "as_int")]
        pub enum Type00Case4PropColourTemp {
            OutOfRange = 16382,
            NoChange = 16383,
            _Other(u16),
        }
        /// Products with Multisensor and/or Light Units (D2-41-00), case 5
        #[derive(Clone, Copy)]
        pub struct Type00Case5<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case5<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Unit Index value.
            pub fn get_unit_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Unit Index value.
            pub fn get_unit_index(&self) -> Option<Type00Case5PropUnitIndex> {
                let raw_value = self.get_unit_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case5PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Operating State value.
            pub fn get_operating_state_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 4)
            }
            /// Get the Operating State value.
            pub fn get_operating_state(&self) -> Option<Type00Case5PropOperatingState> {
                let raw_value = self.get_operating_state_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Switch Control value.
            pub fn get_switch_control_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(12, 4)
            }
            /// Get the Switch Control value.
            pub fn get_switch_control(&self) -> Option<Type00Case5PropSwitchControl> {
                let raw_value = self.get_switch_control_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw VTL value.
            pub fn get_vtl_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 4)
            }
            /// Get the VTL value.
            pub fn get_vtl(&self) -> Option<Type00Case5PropVtl> {
                let raw_value = self.get_vtl_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Fading Mode value.
            pub fn get_fading_mode_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(20, 4)
            }
            /// Get the Fading Mode value.
            pub fn get_fading_mode(&self) -> Option<Type00Case5PropFadingMode> {
                let raw_value = self.get_fading_mode_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Dimming Level value.
            pub fn get_dimming_level_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 8)
            }
            /// Get the Dimming Level value.
            pub fn get_dimming_level(&self) -> Option<Type00Case5PropDimmingLevel> {
                let raw_value = self.get_dimming_level_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Colour Temp value.
            pub fn get_colour_temp_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(34, 14)
            }
            /// Get the Colour Temp value.
            pub fn get_colour_temp(&self) -> Option<Type00Case5PropColourTemp> {
                let raw_value = self.get_colour_temp_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case5<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case5")
                    .field("unit_index", &self.get_unit_index())
                    .field("command_id", &self.get_command_id())
                    .field("operating_state", &self.get_operating_state())
                    .field("switch_control", &self.get_switch_control())
                    .field("vtl", &self.get_vtl())
                    .field("fading_mode", &self.get_fading_mode())
                    .field("dimming_level", &self.get_dimming_level())
                    .field("colour_temp", &self.get_colour_temp())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case5PropUnitIndex {
            NoUnitConnected = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case5PropCommandId {
            UnitStatus = 9,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case5PropOperatingState {
            Off = 0,
            ReducedLight = 1,
            WorkingLight = 2,
            ServiceLight = 3,
            NotSupported = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case5PropSwitchControl {
            NotUsed = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case5PropVtl {
            Off = 0,
            Normal = 1,
            Owl = 2,
            Lark = 3,
            NotSupported = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case5PropFadingMode {
            NotUsed = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case5PropDimmingLevel {
            NotSupported = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u16, derive_compare = "as_int")]
        pub enum Type00Case5PropColourTemp {
            NotSupported = 16383,
            _Other(u16),
        }
        /// Products with Multisensor and/or Light Units (D2-41-00), case 6
        #[derive(Clone, Copy)]
        pub struct Type00Case6<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case6<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Unit Index value.
            pub fn get_unit_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Unit Index value.
            pub fn get_unit_index(&self) -> Option<Type00Case6PropUnitIndex> {
                let raw_value = self.get_unit_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case6PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Occupancy value.
            pub fn get_occupancy_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 3)
            }
            /// Get the Occupancy value.
            pub fn get_occupancy(&self) -> Option<Type00Case6PropOccupancy> {
                let raw_value = self.get_occupancy_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Occupancy 1 value.
            pub fn get_occupancy_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(11, 3)
            }
            /// Get the Occupancy 1 value.
            pub fn get_occupancy_1(&self) -> Option<Type00Case6PropOccupancy1> {
                let raw_value = self.get_occupancy_1_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case6<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case6")
                    .field("unit_index", &self.get_unit_index())
                    .field("command_id", &self.get_command_id())
                    .field("occupancy", &self.get_occupancy())
                    .field("occupancy_1", &self.get_occupancy_1())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case6PropUnitIndex {
            Reserved = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case6PropCommandId {
            SetOccupancy = 10,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case6PropOccupancy {
            NotOccupied = 0,
            Occupied = 1,
            NoChange = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case6PropOccupancy1 {
            NotOccupied = 0,
            Occupied = 1,
            NoChange = 7,
            _Other(u8),
        }
        /// Products with Multisensor and/or Light Units (D2-41-00), case 7
        #[derive(Clone, Copy)]
        pub struct Type00Case7<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case7<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Unit Index value.
            pub fn get_unit_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Unit Index value.
            pub fn get_unit_index(&self) -> Option<Type00Case7PropUnitIndex> {
                let raw_value = self.get_unit_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case7PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Occupancy value.
            pub fn get_occupancy_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 3)
            }
            /// Get the Occupancy value.
            pub fn get_occupancy(&self) -> Option<Type00Case7PropOccupancy> {
                let raw_value = self.get_occupancy_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Occupancy 1 value.
            pub fn get_occupancy_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(11, 3)
            }
            /// Get the Occupancy 1 value.
            pub fn get_occupancy_1(&self) -> Option<Type00Case7PropOccupancy1> {
                let raw_value = self.get_occupancy_1_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Presence value.
            pub fn get_presence_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(14, 2)
            }
            /// Get the Presence value.
            pub fn get_presence(&self) -> Option<Type00Case7PropPresence> {
                let raw_value = self.get_presence_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case7<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case7")
                    .field("unit_index", &self.get_unit_index())
                    .field("command_id", &self.get_command_id())
                    .field("occupancy", &self.get_occupancy())
                    .field("occupancy_1", &self.get_occupancy_1())
                    .field("presence", &self.get_presence())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case7PropUnitIndex {
            NoUnitConnected = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case7PropCommandId {
            PresenceData = 11,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case7PropOccupancy {
            NotOccupied = 0,
            Occupied = 1,
            NotSupported = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case7PropOccupancy1 {
            NotOccupied = 0,
            Occupied = 1,
            NotSupported = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case7PropPresence {
            NoPresenceDetected = 0,
            PresenceDetected = 1,
            Reserved = 2,
            NotSupported = 3,
            _Other(u8),
        }
        /// Products with Multisensor and/or Light Units (D2-41-00), case 8
        #[derive(Clone, Copy)]
        pub struct Type00Case8<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case8<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Unit Index value.
            pub fn get_unit_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Unit Index value.
            pub fn get_unit_index(&self) -> Option<Type00Case8PropUnitIndex> {
                let raw_value = self.get_unit_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case8PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Noise Level value.
            pub fn get_noise_level_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Noise Level value.
            pub fn get_noise_level(&self) -> Option<Type00Case8PropNoiseLevel> {
                let raw_value = self.get_noise_level_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw VOC value.
            pub fn get_voc_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(16, 16)
            }
            /// Get the VOC value.
            pub fn get_voc(&self) -> Option<Type00Case8PropVoc> {
                let raw_value = self.get_voc_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Illumination value.
            pub fn get_illumination_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(33, 15)
            }
            /// Get the Illumination value.
            pub fn get_illumination(&self) -> Option<Type00Case8PropIllumination> {
                let raw_value = self.get_illumination_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(49, 7)
            }
            /// Get the Temperature value.
            pub fn get_temperature(&self) -> Option<Type00Case8PropTemperature> {
                let raw_value = self.get_temperature_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Humidity value.
            pub fn get_humidity_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(57, 7)
            }
            /// Get the Humidity value.
            pub fn get_humidity(&self) -> Option<Type00Case8PropHumidity> {
                let raw_value = self.get_humidity_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case8<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case8")
                    .field("unit_index", &self.get_unit_index())
                    .field("command_id", &self.get_command_id())
                    .field("noise_level", &self.get_noise_level())
                    .field("voc", &self.get_voc())
                    .field("illumination", &self.get_illumination())
                    .field("temperature", &self.get_temperature())
                    .field("humidity", &self.get_humidity())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case8PropUnitIndex {
            Reserved = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case8PropCommandId {
            EnvironmentalData = 12,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case8PropNoiseLevel {
            NotSupported = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u16, derive_compare = "as_int")]
        pub enum Type00Case8PropVoc {
            NotSupported = 65535,
            _Other(u16),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u16, derive_compare = "as_int")]
        pub enum Type00Case8PropIllumination {
            NotSupported = 32767,
            _Other(u16),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case8PropTemperature {
            NotSupported = 127,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case8PropHumidity {
            NotSupported = 127,
            _Other(u8),
        }
        /// Products with Multisensor and/or Light Units (D2-41-00), case 9
        #[derive(Clone, Copy)]
        pub struct Type00Case9<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case9<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Unit Index value.
            pub fn get_unit_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 4)
            }
            /// Get the Unit Index value.
            pub fn get_unit_index(&self) -> Option<Type00Case9PropUnitIndex> {
                let raw_value = self.get_unit_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Command ID value.
            pub fn get_command_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Command ID value.
            pub fn get_command_id(&self) -> Option<Type00Case9PropCommandId> {
                let raw_value = self.get_command_id_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Operating Hours value.
            pub fn get_operating_hours_raw(&self) -> Option<u32> {
                self.reversed_bytes.u32_from_bits(8, 18)
            }
            /// Get the Operating Hours value.
            pub fn get_operating_hours(&self) -> Option<Type00Case9PropOperatingHours> {
                let raw_value = self.get_operating_hours_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Operating Hours Active value.
            pub fn get_operating_hours_active_raw(&self) -> Option<u32> {
                self.reversed_bytes.u32_from_bits(26, 18)
            }
            /// Get the Operating Hours Active value.
            pub fn get_operating_hours_active(&self) -> Option<Type00Case9PropOperatingHoursActive> {
                let raw_value = self.get_operating_hours_active_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Power Consumption value.
            pub fn get_power_consumption_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(44, 12)
            }
            /// Get the Power Consumption value.
            pub fn get_power_consumption(&self) -> Option<Type00Case9PropPowerConsumption> {
                let raw_value = self.get_power_consumption_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Energy Consumption value.
            pub fn get_energy_consumption_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(56, 16)
            }
            /// Get the Energy Consumption value.
            pub fn get_energy_consumption(&self) -> Option<Type00Case9PropEnergyConsumption> {
                let raw_value = self.get_energy_consumption_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case9<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case9")
                    .field("unit_index", &self.get_unit_index())
                    .field("command_id", &self.get_command_id())
                    .field("operating_hours", &self.get_operating_hours())
                    .field("operating_hours_active", &self.get_operating_hours_active())
                    .field("power_consumption", &self.get_power_consumption())
                    .field("energy_consumption", &self.get_energy_consumption())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case9PropUnitIndex {
            NoUnitConnected = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case9PropCommandId {
            MaintenanceData = 13,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u32, derive_compare = "as_int")]
        pub enum Type00Case9PropOperatingHours {
            OutOfRange = 262142,
            NotSupported = 262143,
            _Other(u32),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u32, derive_compare = "as_int")]
        pub enum Type00Case9PropOperatingHoursActive {
            OutOfRange = 262142,
            NotSupported = 262143,
            _Other(u32),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u16, derive_compare = "as_int")]
        pub enum Type00Case9PropPowerConsumption {
            OutOfRange = 4094,
            NotSupported = 4095,
            _Other(u16),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u16, derive_compare = "as_int")]
        pub enum Type00Case9PropEnergyConsumption {
            OutOfRange = 65534,
            NotSupported = 65535,
            _Other(u16),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func41<'b> {
        Type00Case0(func41::Type00Case0<'b>),
        Type00Case1(func41::Type00Case1<'b>),
        Type00Case2(func41::Type00Case2<'b>),
        Type00Case3(func41::Type00Case3<'b>),
        Type00Case4(func41::Type00Case4<'b>),
        Type00Case5(func41::Type00Case5<'b>),
        Type00Case6(func41::Type00Case6<'b>),
        Type00Case7(func41::Type00Case7<'b>),
        Type00Case8(func41::Type00Case8<'b>),
        Type00Case9(func41::Type00Case9<'b>),
    }
    impl<'b> Func41<'b> {
        pub fn type00_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 10> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type00Case0(
                func41::Type00Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case1(
                func41::Type00Case1::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case2(
                func41::Type00Case2::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case3(
                func41::Type00Case3::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case4(
                func41::Type00Case4::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case5(
                func41::Type00Case5::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case6(
                func41::Type00Case6::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case7(
                func41::Type00Case7::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case8(
                func41::Type00Case8::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case9(
                func41::Type00Case9::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 10>> {
            match type_code {
                0x00 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type00_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Heat Recovery Ventilation (D2-50)
    pub mod func50 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Type 0x00 (D2-50-00), case 0
        #[derive(Clone, Copy)]
        pub struct Type00Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Message Type value.
            pub fn get_message_type_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 3)
            }
            /// Get the Message Type value.
            pub fn get_message_type(&self) -> Option<Type00Case0PropMessageType> {
                let raw_value = self.get_message_type_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Requested Message Type value.
            pub fn get_requested_message_type_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(5, 3)
            }
            /// Get the Requested Message Type value.
            pub fn get_requested_message_type(&self) -> Option<Type00Case0PropRequestedMessageType> {
                let raw_value = self.get_requested_message_type_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case0")
                    .field("message_type", &self.get_message_type())
                    .field("requested_message_type", &self.get_requested_message_type())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case0PropMessageType {
            VentilationRemoteTransmissionRequest = 0,
            VentilationControl = 1,
            VentilationBasicStatus = 2,
            VentilationExtendedStatus = 3,
            Reserved = 4,
            Reserved1 = 5,
            Reserved2 = 6,
            Reserved3 = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case0PropRequestedMessageType {
            VentilationBasicStatus = 0,
            VentilationExtendedStatus = 1,
            Reserved = 2,
            Reserved1 = 3,
            Reserved2 = 4,
            Reserved3 = 5,
            Reserved4 = 6,
            Reserved5 = 7,
            _Other(u8),
        }
        /// Type 0x00 (D2-50-00), case 1
        #[derive(Clone, Copy)]
        pub struct Type00Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Message Type value.
            pub fn get_message_type_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 3)
            }
            /// Get the Message Type value.
            pub fn get_message_type(&self) -> Option<Type00Case1PropMessageType> {
                let raw_value = self.get_message_type_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Direct Operation Mode Control value.
            pub fn get_direct_operation_mode_control_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Direct Operation Mode Control value.
            pub fn get_direct_operation_mode_control(&self) -> Option<Type00Case1PropDirectOperationModeControl> {
                let raw_value = self.get_direct_operation_mode_control_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Operation Mode Control value.
            pub fn get_operation_mode_control_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 2)
            }
            /// Get the Operation Mode Control value.
            pub fn get_operation_mode_control(&self) -> Option<Type00Case1PropOperationModeControl> {
                let raw_value = self.get_operation_mode_control_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Heat Exchanger Bypass Control value.
            pub fn get_heat_exchanger_bypass_control_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(10, 2)
            }
            /// Get the Heat Exchanger Bypass Control value.
            pub fn get_heat_exchanger_bypass_control(&self) -> Option<Type00Case1PropHeatExchangerBypassControl> {
                let raw_value = self.get_heat_exchanger_bypass_control_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Timer Operation Mode Control value.
            pub fn get_timer_operation_mode_control_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(16, 1)
            }
            /// Get the Timer Operation Mode Control value.
            pub fn get_timer_operation_mode_control(&self) -> Option<Type00Case1PropTimerOperationModeControl> {
                let raw_value = self.get_timer_operation_mode_control_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw CO2 Threshold value.
            pub fn get_co2_threshold_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(17, 7)
            }
            /// Get the CO2 Threshold value.
            pub fn get_co2_threshold(&self) -> Option<Type00Case1PropCo2Threshold> {
                let raw_value = self.get_co2_threshold_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Humidity Threshold value.
            pub fn get_humidity_threshold_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(25, 7)
            }
            /// Get the Humidity Threshold value.
            pub fn get_humidity_threshold(&self) -> Option<Type00Case1PropHumidityThreshold> {
                let raw_value = self.get_humidity_threshold_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Air Quality Threshold value.
            pub fn get_air_quality_threshold_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(33, 7)
            }
            /// Get the Air Quality Threshold value.
            pub fn get_air_quality_threshold(&self) -> Option<Type00Case1PropAirQualityThreshold> {
                let raw_value = self.get_air_quality_threshold_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Room temperature threshold value.
            pub fn get_room_temperature_threshold_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(41, 7)
            }
            /// Get the Room temperature threshold value.
            pub fn get_room_temperature_threshold(&self) -> Option<Type00Case1PropRoomTemperatureThreshold> {
                let raw_value = self.get_room_temperature_threshold_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case1")
                    .field("message_type", &self.get_message_type())
                    .field("direct_operation_mode_control", &self.get_direct_operation_mode_control())
                    .field("operation_mode_control", &self.get_operation_mode_control())
                    .field("heat_exchanger_bypass_control", &self.get_heat_exchanger_bypass_control())
                    .field("timer_operation_mode_control", &self.get_timer_operation_mode_control())
                    .field("co2_threshold", &self.get_co2_threshold())
                    .field("humidity_threshold", &self.get_humidity_threshold())
                    .field("air_quality_threshold", &self.get_air_quality_threshold())
                    .field("room_temperature_threshold", &self.get_room_temperature_threshold())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropMessageType {
            VentilationRemoteTransmissionRequest = 0,
            VentilationControl = 1,
            VentilationBasicStatus = 2,
            VentilationExtendedStatus = 3,
            Reserved = 4,
            Reserved1 = 5,
            Reserved2 = 6,
            Reserved3 = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropDirectOperationModeControl {
            Off = 0,
            Level1 = 1,
            Level2 = 2,
            Level3 = 3,
            Level4 = 4,
            Reserved = 5,
            Reserved1 = 6,
            Reserved2 = 7,
            Reserved3 = 8,
            Reserved4 = 9,
            Reserved5 = 10,
            Automatic = 11,
            AutomaticOnDemand = 12,
            SupplyAirOnly = 13,
            ExhaustAirOnly = 14,
            NoActionKeepCurrentVentilationModeLevel = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropOperationModeControl {
            NoAction = 0,
            SelectNextOperationModeEdgeTrigger = 1,
            SelectPreviousOperationModeEdgeTrigger = 2,
            Reserved = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropHeatExchangerBypassControl {
            NoAction = 0,
            CloseBypassEdgeTrigger = 1,
            OpenBypassEdgeTrigger = 2,
            Reserved = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case1PropTimerOperationModeControl {
            NoAction = false,
            StartTimerOperationModeEdgeTrigger = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropCo2Threshold {
            DefaultUseThresholdConfiguredInDevice = 127,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropHumidityThreshold {
            DefaultUseThresholdConfiguredInDevice = 127,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropAirQualityThreshold {
            DefaultUseThresholdConfiguredInDevice = 127,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case1PropRoomTemperatureThreshold {
            DefaultUseThresholdConfiguredInDevice = 0,
            _Other(u8),
        }
        /// Type 0x00 (D2-50-00), case 2
        #[derive(Clone, Copy)]
        pub struct Type00Case2<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case2<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Message Type value.
            pub fn get_message_type_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 3)
            }
            /// Get the Message Type value.
            pub fn get_message_type(&self) -> Option<Type00Case2PropMessageType> {
                let raw_value = self.get_message_type_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Operation Mode Status value.
            pub fn get_operation_mode_status_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(4, 4)
            }
            /// Get the Operation Mode Status value.
            pub fn get_operation_mode_status(&self) -> Option<Type00Case2PropOperationModeStatus> {
                let raw_value = self.get_operation_mode_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Safety Mode Status value.
            pub fn get_safety_mode_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(12, 1)
            }
            /// Get the Safety Mode Status value.
            pub fn get_safety_mode_status(&self) -> Option<Type00Case2PropSafetyModeStatus> {
                let raw_value = self.get_safety_mode_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Heat Exchanger Bypass Status value.
            pub fn get_heat_exchanger_bypass_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(13, 1)
            }
            /// Get the Heat Exchanger Bypass Status value.
            pub fn get_heat_exchanger_bypass_status(&self) -> Option<Type00Case2PropHeatExchangerBypassStatus> {
                let raw_value = self.get_heat_exchanger_bypass_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply Air Flap Position value.
            pub fn get_supply_air_flap_position_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(14, 1)
            }
            /// Get the Supply Air Flap Position value.
            pub fn get_supply_air_flap_position(&self) -> Option<Type00Case2PropSupplyAirFlapPosition> {
                let raw_value = self.get_supply_air_flap_position_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Exhaust Air Flap Position value.
            pub fn get_exhaust_air_flap_position_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(15, 1)
            }
            /// Get the Exhaust Air Flap Position value.
            pub fn get_exhaust_air_flap_position(&self) -> Option<Type00Case2PropExhaustAirFlapPosition> {
                let raw_value = self.get_exhaust_air_flap_position_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Defrost Mode Status value.
            pub fn get_defrost_mode_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(16, 1)
            }
            /// Get the Defrost Mode Status value.
            pub fn get_defrost_mode_status(&self) -> Option<Type00Case2PropDefrostModeStatus> {
                let raw_value = self.get_defrost_mode_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Cooling Protection Status value.
            pub fn get_cooling_protection_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(17, 1)
            }
            /// Get the Cooling Protection Status value.
            pub fn get_cooling_protection_status(&self) -> Option<Type00Case2PropCoolingProtectionStatus> {
                let raw_value = self.get_cooling_protection_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Outdoor Air Heater Status value.
            pub fn get_outdoor_air_heater_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(18, 1)
            }
            /// Get the Outdoor Air Heater Status value.
            pub fn get_outdoor_air_heater_status(&self) -> Option<Type00Case2PropOutdoorAirHeaterStatus> {
                let raw_value = self.get_outdoor_air_heater_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Supply Air Heater Status value.
            pub fn get_supply_air_heater_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(19, 1)
            }
            /// Get the Supply Air Heater Status value.
            pub fn get_supply_air_heater_status(&self) -> Option<Type00Case2PropSupplyAirHeaterStatus> {
                let raw_value = self.get_supply_air_heater_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Drain Heater Status value.
            pub fn get_drain_heater_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(20, 1)
            }
            /// Get the Drain Heater Status value.
            pub fn get_drain_heater_status(&self) -> Option<Type00Case2PropDrainHeaterStatus> {
                let raw_value = self.get_drain_heater_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Timer Operation Mode Status value.
            pub fn get_timer_operation_mode_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(21, 1)
            }
            /// Get the Timer Operation Mode Status value.
            pub fn get_timer_operation_mode_status(&self) -> Option<Type00Case2PropTimerOperationModeStatus> {
                let raw_value = self.get_timer_operation_mode_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Filter Maintenance Status value.
            pub fn get_filter_maintenance_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(22, 1)
            }
            /// Get the Filter Maintenance Status value.
            pub fn get_filter_maintenance_status(&self) -> Option<Type00Case2PropFilterMaintenanceStatus> {
                let raw_value = self.get_filter_maintenance_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Weekly Timer Program Status value.
            pub fn get_weekly_timer_program_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(23, 1)
            }
            /// Get the Weekly Timer Program Status value.
            pub fn get_weekly_timer_program_status(&self) -> Option<Type00Case2PropWeeklyTimerProgramStatus> {
                let raw_value = self.get_weekly_timer_program_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Room Temperature Control Status value.
            pub fn get_room_temperature_control_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(24, 1)
            }
            /// Get the Room Temperature Control Status value.
            pub fn get_room_temperature_control_status(&self) -> Option<Type00Case2PropRoomTemperatureControlStatus> {
                let raw_value = self.get_room_temperature_control_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Air Quality Sensor 1 value.
            pub fn get_air_quality_sensor_1_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(25, 7)
            }
            /// Get the Air Quality Sensor 1 value.
            pub fn get_air_quality_sensor_1(&self) -> Option<Type00Case2PropAirQualitySensor1> {
                let raw_value = self.get_air_quality_sensor_1_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Master/Slave Configuration Status value.
            pub fn get_master_slave_configuration_status_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(32, 1)
            }
            /// Get the Master/Slave Configuration Status value.
            pub fn get_master_slave_configuration_status(&self) -> Option<Type00Case2PropMasterSlaveConfigurationStatus> {
                let raw_value = self.get_master_slave_configuration_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Air Quality Sensor 2 value.
            pub fn get_air_quality_sensor_2_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(33, 7)
            }
            /// Get the Air Quality Sensor 2 value.
            pub fn get_air_quality_sensor_2(&self) -> Option<Type00Case2PropAirQualitySensor2> {
                let raw_value = self.get_air_quality_sensor_2_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Outdoor Air Temperature value.
            pub fn get_outdoor_air_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(40, 7)
            }
            /// Get the Outdoor Air Temperature value in units of °C.
            pub fn get_outdoor_air_temperature(&self) -> Option<f64> {
                let raw_value = self.get_outdoor_air_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 127.0, -64.0, 63.0))
            }

            /// Get the raw Supply Air Temperature value.
            pub fn get_supply_air_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(47, 7)
            }
            /// Get the Supply Air Temperature value in units of °C.
            pub fn get_supply_air_temperature(&self) -> Option<f64> {
                let raw_value = self.get_supply_air_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 127.0, -64.0, 63.0))
            }

            /// Get the raw Indoor Air Temperature value.
            pub fn get_indoor_air_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(54, 7)
            }
            /// Get the Indoor Air Temperature value in units of °C.
            pub fn get_indoor_air_temperature(&self) -> Option<f64> {
                let raw_value = self.get_indoor_air_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 127.0, -64.0, 63.0))
            }

            /// Get the raw Exhaust Air Temperature value.
            pub fn get_exhaust_air_temperature_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(61, 7)
            }
            /// Get the Exhaust Air Temperature value in units of °C.
            pub fn get_exhaust_air_temperature(&self) -> Option<f64> {
                let raw_value = self.get_exhaust_air_temperature_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 127.0, -64.0, 63.0))
            }

            /// Get the raw Supply Air Fan Air Flow Rate value.
            pub fn get_supply_air_fan_air_flow_rate_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(68, 10)
            }
            /// Get the Supply Air Fan Air Flow Rate value in units of m3/h.
            pub fn get_supply_air_fan_air_flow_rate(&self) -> Option<f64> {
                let raw_value = self.get_supply_air_fan_air_flow_rate_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 1023.0, 0.0, 1023.0))
            }

            /// Get the raw Exhaust Air Fan Air Flow Rate value.
            pub fn get_exhaust_air_fan_air_flow_rate_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(78, 10)
            }
            /// Get the Exhaust Air Fan Air Flow Rate value in units of m3/h.
            pub fn get_exhaust_air_fan_air_flow_rate(&self) -> Option<f64> {
                let raw_value = self.get_exhaust_air_fan_air_flow_rate_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 1023.0, 0.0, 1023.0))
            }

            /// Get the raw Supply Fan Speed value.
            pub fn get_supply_fan_speed_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(88, 12)
            }
            /// Get the Supply Fan Speed value in units of 1/min.
            pub fn get_supply_fan_speed(&self) -> Option<f64> {
                let raw_value = self.get_supply_fan_speed_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 4095.0, 0.0, 4095.0))
            }

            /// Get the raw Exhaust Fan Speed value.
            pub fn get_exhaust_fan_speed_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(100, 12)
            }
            /// Get the Exhaust Fan Speed value in units of 1/min.
            pub fn get_exhaust_fan_speed(&self) -> Option<f64> {
                let raw_value = self.get_exhaust_fan_speed_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 4095.0, 0.0, 4095.0))
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case2<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case2")
                    .field("message_type", &self.get_message_type())
                    .field("operation_mode_status", &self.get_operation_mode_status())
                    .field("safety_mode_status", &self.get_safety_mode_status())
                    .field("heat_exchanger_bypass_status", &self.get_heat_exchanger_bypass_status())
                    .field("supply_air_flap_position", &self.get_supply_air_flap_position())
                    .field("exhaust_air_flap_position", &self.get_exhaust_air_flap_position())
                    .field("defrost_mode_status", &self.get_defrost_mode_status())
                    .field("cooling_protection_status", &self.get_cooling_protection_status())
                    .field("outdoor_air_heater_status", &self.get_outdoor_air_heater_status())
                    .field("supply_air_heater_status", &self.get_supply_air_heater_status())
                    .field("drain_heater_status", &self.get_drain_heater_status())
                    .field("timer_operation_mode_status", &self.get_timer_operation_mode_status())
                    .field("filter_maintenance_status", &self.get_filter_maintenance_status())
                    .field("weekly_timer_program_status", &self.get_weekly_timer_program_status())
                    .field("room_temperature_control_status", &self.get_room_temperature_control_status())
                    .field("air_quality_sensor_1", &self.get_air_quality_sensor_1())
                    .field("master_slave_configuration_status", &self.get_master_slave_configuration_status())
                    .field("air_quality_sensor_2", &self.get_air_quality_sensor_2())
                    .field("outdoor_air_temperature", &self.get_outdoor_air_temperature())
                    .field("supply_air_temperature", &self.get_supply_air_temperature())
                    .field("indoor_air_temperature", &self.get_indoor_air_temperature())
                    .field("exhaust_air_temperature", &self.get_exhaust_air_temperature())
                    .field("supply_air_fan_air_flow_rate", &self.get_supply_air_fan_air_flow_rate())
                    .field("exhaust_air_fan_air_flow_rate", &self.get_exhaust_air_fan_air_flow_rate())
                    .field("supply_fan_speed", &self.get_supply_fan_speed())
                    .field("exhaust_fan_speed", &self.get_exhaust_fan_speed())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropMessageType {
            VentilationRemoteTransmissionRequest = 0,
            VentilationControl = 1,
            VentilationBasicStatus = 2,
            VentilationExtendedStatus = 3,
            Reserved = 4,
            Reserved1 = 5,
            Reserved2 = 6,
            Reserved3 = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropOperationModeStatus {
            Off = 0,
            Level1 = 1,
            Level2 = 2,
            Level3 = 3,
            Level4 = 4,
            Reserved = 5,
            Reserved1 = 6,
            Reserved2 = 7,
            Reserved3 = 8,
            Reserved4 = 9,
            Reserved5 = 10,
            Automatic = 11,
            AutomaticOnDemand = 12,
            SupplyAirOnly = 13,
            ExhaustAirOnly = 14,
            Reserved6 = 15,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropSafetyModeStatus {
            FireplaceSafetyModeDisabled = false,
            FireplaceSafetyModeEnabled = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropHeatExchangerBypassStatus {
            BypassClosedHeatRecoveryActive = false,
            BypassOpenedHeatRecoveryInactive = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropSupplyAirFlapPosition {
            SupplyAirFlapClosed = false,
            SupplyAirFlapOpened = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropExhaustAirFlapPosition {
            ExhaustAirFlapClosed = false,
            ExhaustAirFlapOpened = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropDefrostModeStatus {
            DefrostModeInactive = false,
            DefrostModeActive = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropCoolingProtectionStatus {
            CoolingProtectionModeInactive = false,
            CoolingProtectionModeActive = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropOutdoorAirHeaterStatus {
            Inactive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropSupplyAirHeaterStatus {
            Inactive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropDrainHeaterStatus {
            Inactive = false,
            Active = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropTimerOperationModeStatus {
            TimerOperationModeInactive = false,
            TimerOperationModeActive = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropFilterMaintenanceStatus {
            MaintenanceNotRequired = false,
            MaintenanceRequired = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropWeeklyTimerProgramStatus {
            WeeklyTimerProgramDisabledOrNotConfigured = false,
            WeeklyTimerProgramActive = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropRoomTemperatureControlStatus {
            RoomTemperatureControlInactive = false,
            RoomTemperatureControlActive = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropAirQualitySensor1 {
            NotAvailable = 127,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type00Case2PropMasterSlaveConfigurationStatus {
            Master = false,
            Slave = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case2PropAirQualitySensor2 {
            NotAvailable = 127,
            _Other(u8),
        }
        /// Type 0x00 (D2-50-00), case 3
        #[derive(Clone, Copy)]
        pub struct Type00Case3<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00Case3<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Message Type value.
            pub fn get_message_type_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 3)
            }
            /// Get the Message Type value.
            pub fn get_message_type(&self) -> Option<Type00Case3PropMessageType> {
                let raw_value = self.get_message_type_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Software Version Info value.
            pub fn get_software_version_info_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(4, 12)
            }
            /// Get the Software Version Info value in units of -.
            pub fn get_software_version_info(&self) -> Option<f64> {
                let raw_value = self.get_software_version_info_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 4095.0, 0.0, 4095.0))
            }

            /// Get the raw Operation Hours Counter value.
            pub fn get_operation_hours_counter_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(16, 16)
            }
            /// Get the Operation Hours Counter value in units of h.
            pub fn get_operation_hours_counter(&self) -> Option<f64> {
                let raw_value = self.get_operation_hours_counter_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 65535.0, 0.0, 196605.0))
            }

            /// Get the raw Digital Input 0...15 Status value.
            pub fn get_digital_input_0_15_status_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(32, 16)
            }
            /// Get the Digital Input 0...15 Status value.
            pub fn get_digital_input_0_15_status(&self) -> Option<Type00Case3PropDigitalInput015Status> {
                let raw_value = self.get_digital_input_0_15_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Digital Output 0...15 Status value.
            pub fn get_digital_output_0_15_status_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(48, 16)
            }
            /// Get the Digital Output 0...15 Status value.
            pub fn get_digital_output_0_15_status(&self) -> Option<Type00Case3PropDigitalOutput015Status> {
                let raw_value = self.get_digital_output_0_15_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Info Message 0...15 Status value.
            pub fn get_info_message_0_15_status_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(64, 16)
            }
            /// Get the Info Message 0...15 Status value.
            pub fn get_info_message_0_15_status(&self) -> Option<Type00Case3PropInfoMessage015Status> {
                let raw_value = self.get_info_message_0_15_status_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Fault 0...31 Status value.
            pub fn get_fault_0_31_status_raw(&self) -> Option<u32> {
                self.reversed_bytes.u32_from_bits(80, 32)
            }
            /// Get the Fault 0...31 Status value.
            pub fn get_fault_0_31_status(&self) -> Option<Type00Case3PropFault031Status> {
                let raw_value = self.get_fault_0_31_status_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00Case3<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00Case3")
                    .field("message_type", &self.get_message_type())
                    .field("software_version_info", &self.get_software_version_info())
                    .field("operation_hours_counter", &self.get_operation_hours_counter())
                    .field("digital_input_0_15_status", &self.get_digital_input_0_15_status())
                    .field("digital_output_0_15_status", &self.get_digital_output_0_15_status())
                    .field("info_message_0_15_status", &self.get_info_message_0_15_status())
                    .field("fault_0_31_status", &self.get_fault_0_31_status())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00Case3PropMessageType {
            VentilationRemoteTransmissionRequest = 0,
            VentilationControl = 1,
            VentilationBasicStatus = 2,
            VentilationExtendedStatus = 3,
            Reserved = 4,
            Reserved1 = 5,
            Reserved2 = 6,
            Reserved3 = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u16, derive_compare = "as_int")]
        pub enum Type00Case3PropDigitalInput015Status {
            InputNo00Active = 1,
            InputNo01Active = 2,
            InputNo02Active = 4,
            InputNo03Active = 8,
            InputNo04Active = 16,
            InputNo05Active = 32,
            InputNo06Active = 64,
            InputNo07Active = 128,
            InputNo08Active = 256,
            InputNo09Active = 512,
            InputNo10Active = 1024,
            InputNo11Active = 2048,
            InputNo12Active = 4096,
            InputNo13Active = 8192,
            InputNo14Active = 16384,
            InputNo15Active = 32768,
            _Other(u16),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u16, derive_compare = "as_int")]
        pub enum Type00Case3PropDigitalOutput015Status {
            OutputNo00Active = 1,
            OutputNo01Active = 2,
            OutputNo02Active = 4,
            OutputNo03Active = 8,
            OutputNo04Active = 16,
            OutputNo05Active = 32,
            OutputNo06Active = 64,
            OutputNo07Active = 128,
            OutputNo08Active = 256,
            OutputNo09Active = 512,
            OutputNo10Active = 1024,
            OutputNo11Active = 2048,
            OutputNo12Active = 4096,
            OutputNo13Active = 8192,
            OutputNo14Active = 16384,
            OutputNo15Active = 32768,
            _Other(u16),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u16, derive_compare = "as_int")]
        pub enum Type00Case3PropInfoMessage015Status {
            InfoNo00Active = 1,
            InfoNo01Active = 2,
            InfoNo02Active = 4,
            InfoNo03Active = 8,
            InfoNo04Active = 16,
            InfoNo05Active = 32,
            InfoNo06Active = 64,
            InfoNo07Active = 128,
            InfoNo08Active = 256,
            InfoNo09Active = 512,
            InfoNo10Active = 1024,
            InfoNo11Active = 2048,
            InfoNo12Active = 4096,
            InfoNo13Active = 8192,
            InfoNo14Active = 16384,
            InfoNo15Active = 32768,
            _Other(u16),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u32, derive_compare = "as_int")]
        pub enum Type00Case3PropFault031Status {
            FaultNo00Active = 1,
            FaultNo01Active = 2,
            FaultNo02Active = 4,
            FaultNo03Active = 8,
            FaultNo04Active = 16,
            FaultNo05Active = 32,
            FaultNo06Active = 64,
            FaultNo07Active = 128,
            FaultNo08Active = 256,
            FaultNo09Active = 512,
            FaultNo10Active = 1024,
            FaultNo11Active = 2048,
            FaultNo12Active = 4096,
            FaultNo13Active = 8192,
            FaultNo14Active = 16384,
            FaultNo15Active = 32768,
            FaultNo16Active = 65536,
            FaultNo17Active = 131072,
            FaultNo18Active = 262144,
            FaultNo19Active = 524288,
            FaultNo20Active = 1048576,
            FaultNo21Active = 2097152,
            FaultNo22Active = 4194304,
            FaultNo23Active = 8388608,
            FaultNo24Active = 16777216,
            FaultNo25Active = 33554432,
            FaultNo26Active = 67108864,
            FaultNo27Active = 134217728,
            FaultNo28Active = 268435456,
            FaultNo29Active = 536870912,
            FaultNo30Active = 1073741824,
            FaultNo31Active = 2147483648,
            _Other(u32),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func50<'b> {
        Type00Case0(func50::Type00Case0<'b>),
        Type00Case1(func50::Type00Case1<'b>),
        Type00Case2(func50::Type00Case2<'b>),
        Type00Case3(func50::Type00Case3<'b>),
    }
    impl<'b> Func50<'b> {
        pub fn type00_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 4> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type00Case0(
                func50::Type00Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case1(
                func50::Type00Case1::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case2(
                func50::Type00Case2::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type00Case3(
                func50::Type00Case3::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 4>> {
            match type_code {
                0x00 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type00_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Special applications (D2-60)
    pub mod func60 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Physiological effects indexes (D2-60-00)
        #[derive(Clone, Copy)]
        pub struct Type00<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Cognitivity / Productivity index value.
            pub fn get_cognitivity_productivity_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Cognitivity / Productivity index value.
            pub fn get_cognitivity_productivity_index(&self) -> Option<Type00PropCognitivityProductivityIndex> {
                let raw_value = self.get_cognitivity_productivity_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Long term health index value.
            pub fn get_long_term_health_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Long term health index value.
            pub fn get_long_term_health_index(&self) -> Option<Type00PropLongTermHealthIndex> {
                let raw_value = self.get_long_term_health_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Short term health index value.
            pub fn get_short_term_health_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Short term health index value.
            pub fn get_short_term_health_index(&self) -> Option<Type00PropShortTermHealthIndex> {
                let raw_value = self.get_short_term_health_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Quality of sleep index value.
            pub fn get_quality_of_sleep_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 8)
            }
            /// Get the Quality of sleep index value.
            pub fn get_quality_of_sleep_index(&self) -> Option<Type00PropQualityOfSleepIndex> {
                let raw_value = self.get_quality_of_sleep_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Respiratory tract irritation index value.
            pub fn get_respiratory_tract_irritation_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(32, 8)
            }
            /// Get the Respiratory tract irritation index value.
            pub fn get_respiratory_tract_irritation_index(&self) -> Option<Type00PropRespiratoryTractIrritationIndex> {
                let raw_value = self.get_respiratory_tract_irritation_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Olfactory comfort index value.
            pub fn get_olfactory_comfort_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(40, 8)
            }
            /// Get the Olfactory comfort index value.
            pub fn get_olfactory_comfort_index(&self) -> Option<Type00PropOlfactoryComfortIndex> {
                let raw_value = self.get_olfactory_comfort_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Virus spreading risk index value.
            pub fn get_virus_spreading_risk_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(48, 8)
            }
            /// Get the Virus spreading risk index value.
            pub fn get_virus_spreading_risk_index(&self) -> Option<Type00PropVirusSpreadingRiskIndex> {
                let raw_value = self.get_virus_spreading_risk_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Building health index value.
            pub fn get_building_health_index_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(56, 8)
            }
            /// Get the Building health index value.
            pub fn get_building_health_index(&self) -> Option<Type00PropBuildingHealthIndex> {
                let raw_value = self.get_building_health_index_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Mode value.
            pub fn get_mode_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(70, 2)
            }
            /// Get the Mode value.
            pub fn get_mode(&self) -> Option<Type00PropMode> {
                let raw_value = self.get_mode_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00")
                    .field("cognitivity_productivity_index", &self.get_cognitivity_productivity_index())
                    .field("long_term_health_index", &self.get_long_term_health_index())
                    .field("short_term_health_index", &self.get_short_term_health_index())
                    .field("quality_of_sleep_index", &self.get_quality_of_sleep_index())
                    .field("respiratory_tract_irritation_index", &self.get_respiratory_tract_irritation_index())
                    .field("olfactory_comfort_index", &self.get_olfactory_comfort_index())
                    .field("virus_spreading_risk_index", &self.get_virus_spreading_risk_index())
                    .field("building_health_index", &self.get_building_health_index())
                    .field("mode", &self.get_mode())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropCognitivityProductivityIndex {
            SensorNotPresent = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropLongTermHealthIndex {
            SensorNotPresent = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropShortTermHealthIndex {
            SensorNotPresent = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropQualityOfSleepIndex {
            SensorNotPresent = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropRespiratoryTractIrritationIndex {
            SensorNotPresent = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropOlfactoryComfortIndex {
            SensorNotPresent = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropVirusSpreadingRiskIndex {
            SensorNotPresent = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropBuildingHealthIndex {
            SensorNotPresent = 255,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropMode {
            Comfort = 0,
            Eco = 1,
            Night = 2,
            Maintenace = 3,
            _Other(u8),
        }
        /// IAQ setpoints (D2-60-01)
        #[derive(Clone, Copy)]
        pub struct Type01<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw CO2 setpoint value.
            pub fn get_co2_setpoint_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the CO2 setpoint value.
            pub fn get_co2_setpoint(&self) -> Option<Type01PropCo2Setpoint> {
                let raw_value = self.get_co2_setpoint_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw TVOC setpoint value.
            pub fn get_tvoc_setpoint_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the TVOC setpoint value in units of ppm /mg/m3.
            pub fn get_tvoc_setpoint(&self) -> Option<f64> {
                let raw_value = self.get_tvoc_setpoint_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 2550.0))
            }

            /// Get the raw Humidity setpoint value.
            pub fn get_humidity_setpoint_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 8)
            }
            /// Get the Humidity setpoint value.
            pub fn get_humidity_setpoint(&self) -> Option<Type01PropHumiditySetpoint> {
                let raw_value = self.get_humidity_setpoint_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw PM2.5 setpoint value.
            pub fn get_pm2_5_setpoint_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(24, 8)
            }
            /// Get the PM2.5 setpoint value in units of ug/m3.
            pub fn get_pm2_5_setpoint(&self) -> Option<f64> {
                let raw_value = self.get_pm2_5_setpoint_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 255.0))
            }

            /// Get the raw NOx setpoint value.
            pub fn get_nox_setpoint_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(32, 8)
            }
            /// Get the NOx setpoint value in units of ug/m3.
            pub fn get_nox_setpoint(&self) -> Option<f64> {
                let raw_value = self.get_nox_setpoint_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 255.0))
            }

            /// Get the raw O3 setpoint value.
            pub fn get_o3_setpoint_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(40, 8)
            }
            /// Get the O3 setpoint value in units of ug/m3.
            pub fn get_o3_setpoint(&self) -> Option<f64> {
                let raw_value = self.get_o3_setpoint_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 255.0))
            }

            /// Get the raw Specific VOC setpoint value.
            pub fn get_specific_voc_setpoint_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(48, 8)
            }
            /// Get the Specific VOC setpoint value in units of ug/m3.
            pub fn get_specific_voc_setpoint(&self) -> Option<f64> {
                let raw_value = self.get_specific_voc_setpoint_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 255.0))
            }

            /// Get the raw VOC ID value.
            pub fn get_voc_id_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(56, 8)
            }
            /// Get the VOC ID value.
            pub fn get_voc_id(&self) -> Option<f64> {
                let raw_value = self.get_voc_id_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 255.0))
            }

            /// Get the raw Mode value.
            pub fn get_mode_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(64, 2)
            }
            /// Get the Mode value.
            pub fn get_mode(&self) -> Option<Type01PropMode> {
                let raw_value = self.get_mode_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw TVOC Scale Multiplier value.
            pub fn get_tvoc_scale_multiplier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(66, 3)
            }
            /// Get the TVOC Scale Multiplier value.
            pub fn get_tvoc_scale_multiplier(&self) -> Option<Type01PropTvocScaleMultiplier> {
                let raw_value = self.get_tvoc_scale_multiplier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Specific VOC Scale Multiplier value.
            pub fn get_specific_voc_scale_multiplier_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(69, 3)
            }
            /// Get the Specific VOC Scale Multiplier value.
            pub fn get_specific_voc_scale_multiplier(&self) -> Option<Type01PropSpecificVocScaleMultiplier> {
                let raw_value = self.get_specific_voc_scale_multiplier_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw TVOC unit value.
            pub fn get_tvoc_unit_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(72, 1)
            }
            /// Get the TVOC unit value.
            pub fn get_tvoc_unit(&self) -> Option<Type01PropTvocUnit> {
                let raw_value = self.get_tvoc_unit_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01")
                    .field("co2_setpoint", &self.get_co2_setpoint())
                    .field("tvoc_setpoint", &self.get_tvoc_setpoint())
                    .field("humidity_setpoint", &self.get_humidity_setpoint())
                    .field("pm2_5_setpoint", &self.get_pm2_5_setpoint())
                    .field("nox_setpoint", &self.get_nox_setpoint())
                    .field("o3_setpoint", &self.get_o3_setpoint())
                    .field("specific_voc_setpoint", &self.get_specific_voc_setpoint())
                    .field("voc_id", &self.get_voc_id())
                    .field("mode", &self.get_mode())
                    .field("tvoc_scale_multiplier", &self.get_tvoc_scale_multiplier())
                    .field("specific_voc_scale_multiplier", &self.get_specific_voc_scale_multiplier())
                    .field("tvoc_unit", &self.get_tvoc_unit())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropCo2Setpoint {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropHumiditySetpoint {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropMode {
            Comfort = 0,
            Eco = 1,
            Night = 2,
            Maintenace = 3,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropTvocScaleMultiplier {
            _001 = 0,
            _01 = 1,
            _1 = 2,
            _10 = 3,
            _100 = 4,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01PropSpecificVocScaleMultiplier {
            _001 = 0,
            _01 = 1,
            _1 = 2,
            _10 = 3,
            _100 = 4,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type01PropTvocUnit {
            Ppm = false,
            MgM3 = true,
            _Other(bool),
        }
        /// Damper, valve and VAV returned data (D2-60-02)
        #[derive(Clone, Copy)]
        pub struct Type02<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type02<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Actuator type value.
            pub fn get_actuator_type_raw(&self) -> Option<bool> {
                self.reversed_bytes.bool_from_bits(0, 1)
            }
            /// Get the Actuator type value.
            pub fn get_actuator_type(&self) -> Option<Type02PropActuatorType> {
                let raw_value = self.get_actuator_type_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Actuator function value.
            pub fn get_actuator_function_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(1, 3)
            }
            /// Get the Actuator function value.
            pub fn get_actuator_function(&self) -> Option<Type02PropActuatorFunction> {
                let raw_value = self.get_actuator_function_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Air Flow value.
            pub fn get_air_flow_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(4, 12)
            }
            /// Get the Air Flow value in units of m3h.
            pub fn get_air_flow(&self) -> Option<f64> {
                let raw_value = self.get_air_flow_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 4095.0, 0.0, 4095.0))
            }

            /// Get the raw Opening value.
            pub fn get_opening_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(17, 7)
            }
            /// Get the Opening value.
            pub fn get_opening(&self) -> Option<Type02PropOpening> {
                let raw_value = self.get_opening_raw()?;
                Some(raw_value.into())
            }

            /// Get the raw Temperature value.
            pub fn get_temperature_raw(&self) -> Option<u16> {
                self.reversed_bytes.u16_from_bits(30, 10)
            }
            /// Get the Temperature value.
            pub fn get_temperature(&self) -> Option<Type02PropTemperature> {
                let raw_value = self.get_temperature_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type02<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type02")
                    .field("actuator_type", &self.get_actuator_type())
                    .field("actuator_function", &self.get_actuator_function())
                    .field("air_flow", &self.get_air_flow())
                    .field("opening", &self.get_opening())
                    .field("temperature", &self.get_temperature())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = bool, derive_compare = "as_int")]
        pub enum Type02PropActuatorType {
            DamperOrValve = false,
            VariableAirVolume = true,
            _Other(bool),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type02PropActuatorFunction {
            Unspecified = 0,
            AirExhaust = 1,
            AirSupply = 2,
            HotHydraulic2Ways = 3,
            ColdHydraulic2Ways = 4,
            Seasonal2Ways = 5,
            Hydraulic6Ways = 6,
            Reserved = 7,
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type02PropOpening {
            _Other(u8),
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u16, derive_compare = "as_int")]
        pub enum Type02PropTemperature {
            OutOfRangeNegative = 1001,
            OutOfRangePositive = 1002,
            NotMeasured = 1023,
            _Other(u16),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Func60<'b> {
        Type00(func60::Type00<'b>),
        Type01(func60::Type01<'b>),
        Type02(func60::Type02<'b>),
    }
    impl<'b> Func60<'b> {
        pub fn type00_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type00(
                func60::Type00::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01(
                func60::Type01::new(reversed_bytes)
            )).unwrap();
            ret
        }
        pub fn type02_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type02(
                func60::Type02::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x00 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type00_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                0x02 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type02_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Standard Valve (D2-A0)
    pub mod funcA0 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Valve Control (D2-A0-01), case 0
        #[derive(Clone, Copy)]
        pub struct Type01Case0<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01Case0<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Feedback value.
            pub fn get_feedback_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(6, 2)
            }
            /// Get the Feedback value.
            pub fn get_feedback(&self) -> Option<Type01Case0PropFeedback> {
                let raw_value = self.get_feedback_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01Case0<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01Case0")
                    .field("feedback", &self.get_feedback())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case0PropFeedback {
            NotDefined = 0,
            Closed = 1,
            Opened = 2,
            NotDefined1 = 3,
            _Other(u8),
        }
        /// Valve Control (D2-A0-01), case 1
        #[derive(Clone, Copy)]
        pub struct Type01Case1<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type01Case1<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Request value.
            pub fn get_request_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(6, 2)
            }
            /// Get the Request value.
            pub fn get_request(&self) -> Option<Type01Case1PropRequest> {
                let raw_value = self.get_request_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type01Case1<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type01Case1")
                    .field("request", &self.get_request())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type01Case1PropRequest {
            NoChangeRequestOfFeedback = 0,
            RequestToCloseValve = 1,
            RequestToOpenValve = 2,
            RequestToCloseValve1 = 3,
            _Other(u8),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum FuncA0<'b> {
        Type01Case0(funcA0::Type01Case0<'b>),
        Type01Case1(funcA0::Type01Case1<'b>),
    }
    impl<'b> FuncA0<'b> {
        pub fn type01_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 2> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type01Case0(
                funcA0::Type01Case0::new(reversed_bytes)
            )).unwrap();
            ret.push(Self::Type01Case1(
                funcA0::Type01Case1::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 2>> {
            match type_code {
                0x01 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type01_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Liquid Leakage Sensor (D2-B0)
    pub mod funcB0 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Mechanic Harvester (D2-B0-51)
        #[derive(Clone, Copy)]
        pub struct Type51<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type51<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw Water sensor value.
            pub fn get_water_sensor_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the Water sensor value.
            pub fn get_water_sensor(&self) -> Option<Type51PropWaterSensor> {
                let raw_value = self.get_water_sensor_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type51<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type51")
                    .field("water_sensor", &self.get_water_sensor())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type51PropWaterSensor {
            NoWaterDetectedReturnPosition = 0,
            WaterDetected = 17,
            _Other(u8),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum FuncB0<'b> {
        Type51(funcB0::Type51<'b>),
    }
    impl<'b> FuncB0<'b> {
        pub fn type51_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type51(
                funcB0::Type51::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x51 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type51_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
    /// Level Sensor (D2-B1)
    pub mod funcB1 {
        #[allow(unused)] use super::{BitTwiddling, MaxArray, range_scale};
        /// Dispenser (D2-B1-00)
        #[derive(Clone, Copy)]
        pub struct Type00<'b> {
            reversed_bytes: &'b [u8],
        }
        impl<'b> Type00<'b> {
            pub fn new(reversed_bytes: &'b [u8]) -> Self {
                Self { reversed_bytes }
            }

            /// Get the raw ChannelNumber value.
            pub fn get_channelnumber_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(0, 8)
            }
            /// Get the ChannelNumber value.
            pub fn get_channelnumber(&self) -> Option<f64> {
                let raw_value = self.get_channelnumber_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 255.0))
            }

            /// Get the raw Level of Material value.
            pub fn get_level_of_material_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(8, 8)
            }
            /// Get the Level of Material value.
            pub fn get_level_of_material(&self) -> Option<f64> {
                let raw_value = self.get_level_of_material_raw()? as f64;
                Some(range_scale(raw_value, 0.0, 255.0, 0.0, 100.0))
            }

            /// Get the raw errorCode value.
            pub fn get_errorcode_raw(&self) -> Option<u8> {
                self.reversed_bytes.u8_from_bits(16, 7)
            }
            /// Get the errorCode value.
            pub fn get_errorcode(&self) -> Option<Type00PropErrorcode> {
                let raw_value = self.get_errorcode_raw()?;
                Some(raw_value.into())
            }
        }
        impl<'b> ::core::fmt::Debug for Type00<'b> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Type00")
                    .field("channelnumber", &self.get_channelnumber())
                    .field("level_of_material", &self.get_level_of_material())
                    .field("errorcode", &self.get_errorcode())
                    .finish()
            }
        }
        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        #[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
        pub enum Type00PropErrorcode {
            NoError = 0,
            MeasurementsNotReliable = 1,
            DeviceEmpty = 2,
            LoopLineAtGround = 3,
            MalfunctionOfInternalIrSensor = 4,
            CommunicationErrorWithSensors = 5,
            _Other(u8),
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum FuncB1<'b> {
        Type00(funcB1::Type00<'b>),
    }
    impl<'b> FuncB1<'b> {
        pub fn type00_from_reversed_bytes(reversed_bytes: &'b [u8]) -> MaxArray<Self, 1> {
            let mut ret = MaxArray::new();
            ret.push(Self::Type00(
                funcB1::Type00::new(reversed_bytes)
            )).unwrap();
            ret
        }

        pub fn from_reversed_bytes(type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 1>> {
            match type_code {
                0x00 => {
                    Some(MaxArray::from_iter_or_panic(
                        Self::type00_from_reversed_bytes(reversed_bytes)
                            .iter()
                            .map(|b| *b)
                            .peekable()
                    ))
                },
                _ => None,
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RorgD2<'b> {
    Func00(rorgD2::Func00<'b>),
    Func0A(rorgD2::Func0A<'b>),
    Func15(rorgD2::Func15<'b>),
    Func31(rorgD2::Func31<'b>),
    Func32(rorgD2::Func32<'b>),
    Func33(rorgD2::Func33<'b>),
    Func34(rorgD2::Func34<'b>),
    Func40(rorgD2::Func40<'b>),
    Func41(rorgD2::Func41<'b>),
    Func50(rorgD2::Func50<'b>),
    Func60(rorgD2::Func60<'b>),
    FuncA0(rorgD2::FuncA0<'b>),
    FuncB0(rorgD2::FuncB0<'b>),
    FuncB1(rorgD2::FuncB1<'b>),
}
impl<'b> RorgD2<'b> {
    pub fn from_reversed_bytes(func_code: u8, type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 10>> {
        match func_code {
            0x00 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgD2::Func00::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func00(*f))
                        .peekable()
                ))
            },
            0x0A => {
                Some(MaxArray::from_iter_or_panic(
                    rorgD2::Func0A::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func0A(*f))
                        .peekable()
                ))
            },
            0x15 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgD2::Func15::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func15(*f))
                        .peekable()
                ))
            },
            0x31 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgD2::Func31::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func31(*f))
                        .peekable()
                ))
            },
            0x32 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgD2::Func32::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func32(*f))
                        .peekable()
                ))
            },
            0x33 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgD2::Func33::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func33(*f))
                        .peekable()
                ))
            },
            0x34 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgD2::Func34::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func34(*f))
                        .peekable()
                ))
            },
            0x40 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgD2::Func40::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func40(*f))
                        .peekable()
                ))
            },
            0x41 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgD2::Func41::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func41(*f))
                        .peekable()
                ))
            },
            0x50 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgD2::Func50::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func50(*f))
                        .peekable()
                ))
            },
            0x60 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgD2::Func60::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::Func60(*f))
                        .peekable()
                ))
            },
            0xA0 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgD2::FuncA0::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::FuncA0(*f))
                        .peekable()
                ))
            },
            0xB0 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgD2::FuncB0::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::FuncB0(*f))
                        .peekable()
                ))
            },
            0xB1 => {
                Some(MaxArray::from_iter_or_panic(
                    rorgD2::FuncB1::from_reversed_bytes(type_code, reversed_bytes)?
                        .iter()
                        .map(|f| Self::FuncB1(*f))
                        .peekable()
                ))
            },
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Eep<'b> {
    RorgF6(RorgF6<'b>),
    RorgD5(RorgD5<'b>),
    RorgA5(RorgA5<'b>),
    RorgD2(RorgD2<'b>),
}
impl<'b> Eep<'b> {
    pub fn from_reversed_bytes(rorg_code: u8, func_code: u8, type_code: u8, reversed_bytes: &'b [u8]) -> Option<MaxArray<Self, 10>> {
        match rorg_code {
            0xF6 => {
                Some(MaxArray::from_iter_or_panic(
                    RorgF6::from_reversed_bytes(func_code, type_code, reversed_bytes)?
                        .iter()
                        .map(|r| Self::RorgF6(*r))
                        .peekable()
                ))
            },
            0xD5 => {
                Some(MaxArray::from_iter_or_panic(
                    RorgD5::from_reversed_bytes(func_code, type_code, reversed_bytes)?
                        .iter()
                        .map(|r| Self::RorgD5(*r))
                        .peekable()
                ))
            },
            0xA5 => {
                Some(MaxArray::from_iter_or_panic(
                    RorgA5::from_reversed_bytes(func_code, type_code, reversed_bytes)?
                        .iter()
                        .map(|r| Self::RorgA5(*r))
                        .peekable()
                ))
            },
            0xD2 => {
                Some(MaxArray::from_iter_or_panic(
                    RorgD2::from_reversed_bytes(func_code, type_code, reversed_bytes)?
                        .iter()
                        .map(|r| Self::RorgD2(*r))
                        .peekable()
                ))
            },
            _ => None,
        }
    }
}