use rmk::action::KeyAction;
use rmk::{k, layer, mo};
pub(crate) const COL: usize = 6;
pub(crate) const ROW: usize = 10;
pub(crate) const NUM_LAYER: usize = 3;

#[rustfmt::skip]
pub fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    [
        layer!([
            [k!(Escape), k!(Kc1), k!(Kc2), k!(Kc3), k!(Kc4), k!(Kc5)],
            [k!(Tab), k!(Quote), k!(Comma), k!(Dot), k!(P), k!(Y)],
            [k!(CapsLock), k!(A), k!(O), k!(E), k!(U), k!(I)],
            [k!(LShift), k!(Semicolon), k!(Q), k!(J), k!(K), k!(X)],
            [mo!(1), k!(LCtrl), k!(LAlt), k!(LGui), k!(Enter), k!(No)],
            [k!(Backspace), k!(Kc0), k!(Kc9), k!(Kc8), k!(Kc7), k!(Kc6)],
            [k!(Slash), k!(L), k!(R), k!(C), k!(G), k!(F)],
            [k!(Minus), k!(S), k!(N), k!(T), k!(H), k!(D)],
            [k!(RShift), k!(Z), k!(V), k!(W), k!(M), k!(B)],
            [mo!(2), k!(RCtrl), k!(RAlt), k!(RGui), k!(Space), k!(No)]
        ]),
        layer!([
            [k!(Grave), k!(No), k!(No), k!(No), k!(No), k!(No)],
            [k!(Tab), k!(No), k!(No), k!(No), k!(No), k!(No)],
            [k!(CapsLock), k!(No), k!(No), k!(No), k!(No), k!(No)],
            [k!(LShift), k!(No), k!(No), k!(No), k!(No), k!(No)],
            [mo!(1), k!(LCtrl), k!(LAlt), k!(LGui), k!(Enter), k!(No)],
            [k!(Delete), k!(RightBracket), k!(LeftBracket), k!(No), k!(No), k!(No)],
            [k!(Equal), k!(No), k!(No), k!(No), k!(No), k!(No)],
            [k!(Backslash), k!(No), k!(No), k!(No), k!(No), k!(No)],
            [k!(No), k!(No), k!(No), k!(No), k!(No), k!(No)],
            [mo!(2), k!(RCtrl), k!(RAlt), k!(RGui), k!(Space), k!(No)]
        ]),
        layer!([
            [k!(F1), k!(F2), k!(F3), k!(F4), k!(F5), k!(F6)],
            [k!(Tab), k!(No), k!(No), k!(No), k!(No), k!(No)],
            [k!(CapsLock), k!(No), k!(No), k!(No), k!(No), k!(No)],
            [k!(LShift), k!(No), k!(No), k!(No), k!(No), k!(No)],
            [mo!(1), k!(LCtrl), k!(LAlt), k!(LGui), k!(Enter), k!(No)],
            [k!(F12), k!(F11), k!(F10), k!(F9), k!(F8), k!(F7)],
            [k!(No), k!(End), k!(PageUp), k!(PageDown), k!(Home), k!(No)],
            [k!(No), k!(Right), k!(UP), k!(Down), k!(Left), k!(No)],
            [k!(RShift), k!(No), k!(No), k!(No), k!(No), k!(No)],
            [mo!(2), k!(RCtrl), k!(RAlt), k!(RGui), k!(Space), k!(No)]
        ]),
    ]
}
