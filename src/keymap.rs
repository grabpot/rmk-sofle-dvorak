use rmk::action::KeyAction;
use rmk::{k, layer, mo};
pub(crate) const COL: usize = 6;
pub(crate) const ROW: usize = 10;
pub(crate) const NUM_LAYER: usize = 1;

#[rustfmt::skip]
pub fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    [
        layer!([
            [k!(Escape), k!(Kc1), k!(Kc2), k!(Kc3), k!(Kc4), k!(Kc5)],
            [k!(Tab), k!(Quote), k!(Comma), k!(Dot), k!(P), k!(Y)],
            [k!(CapsLock), k!(A), k!(O), k!(E), k!(U), k!(I)],
            [k!(LShift), k!(Semicolon), k!(Q), k!(J), k!(K), k!(X)],
            [mo!(0), k!(LCtrl), k!(LAlt), k!(LGui), k!(Enter), k!(Y)],
            [k!(Backspace), k!(Kc0), k!(Kc9), k!(Kc8), k!(Kc7), k!(Kc6)],
            [k!(Slash), k!(L), k!(R), k!(C), k!(G), k!(F)],
            [k!(Minus), k!(S), k!(N), k!(T), k!(H), k!(D)],
            [k!(RShift), k!(Z), k!(V), k!(W), k!(M), k!(B)],
            [mo!(0), k!(RCtrl), k!(RAlt), k!(RGui), k!(Space), k!(Y)]
        ]),
    ]
}
