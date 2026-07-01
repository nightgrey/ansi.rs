use crate::ColorSpace;
use maybe::{Maybe, MaybeConst};
use std::fmt::{Debug, Display};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Sub, SubAssign};
use derive_more::Display;

#[derive_const(Eq, Clone, PartialEq, MaybeConst)]
#[derive(Copy, Default, Display)]
#[display("Color::{_variant}")]
pub enum Color {
    /// RGB (0,0,0)
    Black,
    /// RGB (128,0,0)
    Red,
    /// RGB (0,128,0)
    Green,
    /// RGB (128,128,0)
    Yellow,
    /// RGB (0,0,128)
    Blue,
    /// RGB (128,0,128)
    Magenta,
    /// RGB (0,128,128)
    Cyan,
    /// RGB (192,192,192)
    White,
    /// RGB (128,128,128)
    BrightBlack,
    /// RGB (255,0,0)
    BrightRed,
    /// RGB (0,255,0)
    BrightGreen,
    /// RGB (255,255,0)
    BrightYellow,
    /// RGB (0,0,255)
    BrightBlue,
    /// RGB (255,0,255)
    BrightMagenta,
    /// RGB (0,255,255)
    BrightCyan,
    /// RGB (255,255,255)
    BrightWhite,
    // /// RGB (0,0,0)
    // AnotherBlack,
    // /// RGB (0,0,95)
    // StratosBlue,
    // /// RGB (0,0,135)
    // NavyBlue,
    // /// RGB (0,0,175)
    // MidnightBlue,
    // /// RGB (0,0,215)
    // DarkBlue,
    // /// RGB (0,0,255)
    // AnotherBlue,
    // /// RGB (0,95,0)
    // CamaroneGreen,
    // /// RGB (0,95,95)
    // BlueStone,
    // /// RGB (0,95,135)
    // OrientBlue,
    // /// RGB (0,95,175)
    // EndeavourBlue,
    // /// RGB (0,95,215)
    // ScienceBlue,
    // /// RGB (0,95,255)
    // BlueRibbon,
    // /// RGB (0,135,0)
    // JapaneseLaurel,
    // /// RGB (0,135,95)
    // DeepSeaGreen,
    // /// RGB (0,135,135)0
    // Teal,
    // /// RGB (0,135,175)
    // DeepCerulean,
    // /// RGB (0,135,215)
    // LochmaraBlue,
    // /// RGB (0,135,255)
    // AzureRadiance,
    // /// RGB (0,175,0)
    // LightJapaneseLaurel,
    // /// RGB (0,175,95)5
    // Jade,
    // /// RGB (0,175,135)
    // PersianGreen,
    // /// RGB (0,175,175)
    // BondiBlue,
    // /// RGB (0,175,215)
    // Cerulean,
    // /// RGB (0,175,255)
    // LightAzureRadiance,
    // /// RGB (0,215,0)
    // DarkGreen,
    // /// RGB (0,215,95)
    // Malachite,
    // /// RGB (0,215,135)
    // CaribbeanGreen,
    // /// RGB (0,215,175)
    // LightCaribbeanGreen,
    // /// RGB (0,215,215)
    // RobinEggBlue,
    // /// RGB (0,215,255)5
    // Aqua,
    // /// RGB (0,255,0)
    // AnotherGreen,
    // /// RGB (0,255,95)
    // DarkSpringGreen,
    // /// RGB (0,255,135)
    // SpringGreen,
    // /// RGB (0,255,175)
    // LightSpringGreen,
    // /// RGB (0,255,215)
    // BrightTurquoise,
    // /// RGB (0,255,255)
    // AnotherCyan,
    // /// RGB (95,0,0)
    // Rosewood,
    // /// RGB (95,0,95)
    // PompadourMagenta,
    // /// RGB (95,0,135)
    // PigmentIndigo,
    // /// RGB (95,0,175)
    // DarkPurple,
    // /// RGB (95,0,215)
    // ElectricIndigo,
    // /// RGB (95,0,255)
    // ElectricPurple,
    // /// RGB (95,95,0)
    // VerdunGreen,
    // /// RGB (95,95,95)
    // ScorpionOlive,
    // /// RGB (95,95,135)
    // Lilac,
    // /// RGB (95,95,175)
    // ScampiIndigo,
    // /// RGB (95,95,215)
    // Indigo,
    // /// RGB (95,95,255)
    // DarkCornflowerBlue,
    // /// RGB (95,135,0)
    // DarkLimeade,
    // /// RGB (95,135,95)
    // GladeGreen,
    // /// RGB (95,135,135)
    // JuniperGreen,
    // /// RGB (95,135,175)
    // HippieBlue,
    // /// RGB (95,135,215)
    // HavelockBlue,
    // /// RGB (95,135,255)
    // CornflowerBlue,
    // /// RGB (95,175,0)
    // Limeade,
    // /// RGB (95,175,95)
    // FernGreen,
    // /// RGB (95,175,135)
    // SilverTree,
    // /// RGB (95,175,175)
    // Tradewind,
    // /// RGB (95,175,215)
    // ShakespeareBlue,
    // /// RGB (95,175,255)
    // DarkMalibuBlue,
    // /// RGB (95,215,0)
    // DarkBrightGreen,
    // /// RGB (95,215,95)
    // DarkPastelGreen,
    // /// RGB (95,215,135)
    // PastelGreen,
    // /// RGB (95,215,175)
    // DownyTeal,
    // /// RGB (95,215,215)
    // Viking,
    // /// RGB (95,215,255)
    // MalibuBlue,
    // /// RGB (95,255,0)
    // AnotherBrightGreen,
    // /// RGB (95,255,95)
    // DarkScreaminGreen,
    // /// RGB (95,255,135)
    // ScreaminGreen,
    // /// RGB (95,255,175)
    // DarkAquamarine,
    // /// RGB (95,255,215)
    // Aquamarine,
    // /// RGB (95,255,255)
    // LightAquamarine,
    // /// RGB (135,0,0)
    // Maroon,
    // /// RGB (135,0,95)
    // DarkFreshEggplant,
    // /// RGB (135,0,135)
    // LightFreshEggplant,
    // /// RGB (135,0,175)
    // Purple,
    // /// RGB (135,0,215)
    // ElectricViolet,
    // /// RGB (135,0,255)
    // LightElectricViolet,
    // /// RGB (135,95,0)
    // Brown,
    // /// RGB (135,95,95)
    // CopperRose,
    // /// RGB (135,95,135)
    // StrikemasterPurple,
    // /// RGB (135,95,175)
    // DelugePurple,
    // /// RGB (135,95,215)
    // DarkMediumPurple,
    // /// RGB (135,95,255)
    // DarkHeliotropePurple,
    // /// RGB (135,135,0)0
    // Olive,
    // /// RGB (135,135,95)
    // ClayCreekOlive,
    // /// RGB (135,135,135)
    // DarkGray,
    // /// RGB (135,135,175)
    // WildBlueYonder,
    // /// RGB (135,135,215)
    // ChetwodeBlue,
    // /// RGB (135,135,255)
    // SlateBlue,
    // /// RGB (135,175,0)
    // LightLimeade,
    // /// RGB (135,175,95)
    // ChelseaCucumber,
    // /// RGB (135,175,135)
    // BayLeaf,
    // /// RGB (135,175,175)
    // GulfStream,
    // /// RGB (135,175,215)
    // PoloBlue,
    // /// RGB (135,175,255)
    // LightMalibuBlue,
    // /// RGB (135,215,0)
    // Pistachio,
    // /// RGB (135,215,95)
    // LightPastelGreen,
    // /// RGB (135,215,135)
    // DarkFeijoaGreen,
    // /// RGB (135,215,175)
    // VistaBlue,
    // /// RGB (135,215,215)
    // Bermuda,
    // /// RGB (135,215,255)
    // DarkAnakiwaBlue,
    // /// RGB (135,255,0)
    // ChartreuseGreen,
    // /// RGB (135,255,95)
    // LightScreaminGreen,
    // /// RGB (135,255,135)
    // DarkMintGreen,
    // /// RGB (135,255,175)
    // MintGreen,
    // /// RGB (135,255,215)
    // LighterAquamarine,
    // /// RGB (135,255,255)
    // AnakiwaBlue,
    // /// RGB (175,0,0)
    // AnotherBrightRed,
    // /// RGB (175,0,95)
    // DarkFlirt,
    // /// RGB (175,0,135)6
    // Flirt,
    // /// RGB (175,0,175)
    // LightFlirt,
    // /// RGB (175,0,215)
    // DarkViolet,
    // /// RGB (175,0,255)
    // BrightElectricViolet,
    // /// RGB (175,95,0)
    // RoseofSharonOrange,
    // /// RGB (175,95,95)
    // MatrixPink,
    // /// RGB (175,95,135)
    // TapestryPink,
    // /// RGB (175,95,175)
    // FuchsiaPink,
    // /// RGB (175,95,215)
    // MediumPurple,
    // /// RGB (175,95,255)
    // Heliotrope,
    // /// RGB (175,135,0)
    // PirateGold,
    // /// RGB (175,135,95)
    // MuesliOrange,
    // /// RGB (175,135,135)
    // PharlapPink,
    // /// RGB (175,135,175)
    // Bouquet,
    // /// RGB (175,135,215)
    // Lavender,
    // /// RGB (175,135,255)
    // LightHeliotrope,
    // /// RGB (175,175,0)
    // BuddhaGold,
    // /// RGB (175,175,95)
    // OliveGreen,
    // /// RGB (175,175,135)
    // HillaryOlive,
    // /// RGB (175,175,175)
    // SilverChalice,
    // /// RGB (175,175,215)
    // WistfulLilac,
    // /// RGB (175,175,255)
    // MelroseLilac,
    // /// RGB (175,215,0)
    // RioGrandeGreen,
    // /// RGB (175,215,95)
    // ConiferGreen,
    // /// RGB (175,215,135)
    // Feijoa,
    // /// RGB (175,215,175)
    // PixieGreen,
    // /// RGB (175,215,215)
    // JungleMist,
    // /// RGB (175,215,255)
    // LightAnakiwaBlue,
    // /// RGB (175,255,0)54
    // Lime,
    // /// RGB (175,255,95)
    // GreenYellow,
    // /// RGB (175,255,135)
    // LightMintGreen,
    // /// RGB (175,255,175)
    // Celadon,
    // /// RGB (175,255,215)
    // AeroBlue,
    // /// RGB (175,255,255)
    // FrenchPassLightBlue,
    // /// RGB (215,0,0)
    // GuardsmanRed,
    // /// RGB (215,0,95)
    // RazzmatazzCerise,
    // /// RGB (215,0,135)
    // MediumVioletRed,
    // /// RGB (215,0,175)
    // HollywoodCerise,
    // /// RGB (215,0,215)
    // DarkPurplePizzazz,
    // /// RGB (215,0,255)
    // BrighterElectricViolet,
    // /// RGB (215,95,0)
    // TennOrange,
    // /// RGB (215,95,95)
    // RomanOrange,
    // /// RGB (215,95,135)
    // CranberryPink,
    // /// RGB (215,95,175)
    // HopbushPink,
    // /// RGB (215,95,215)
    // Orchid,
    // /// RGB (215,95,255)
    // LighterHeliotrope,
    // /// RGB (215,135,0)
    // MangoTango,
    // /// RGB (215,135,95)
    // Copperfield,
    // /// RGB (215,135,135)
    // SeaPink,
    // /// RGB (215,135,175)
    // CanCanPink,
    // /// RGB (215,135,215)
    // LightOrchid,
    // /// RGB (215,135,255)
    // BrightHeliotrope,
    // /// RGB (215,175,0)
    // DarkCorn,
    // /// RGB (215,175,95)
    // DarkTachaOrange,
    // /// RGB (215,175,135)
    // TanBeige,
    // /// RGB (215,175,175)
    // ClamShell,
    // /// RGB (215,175,215)
    // ThistlePink,
    // /// RGB (215,175,255)
    // Mauve,
    // /// RGB (215,215,0)
    // Corn,
    // /// RGB (215,215,95)
    // TachaOrange,
    // /// RGB (215,215,135)
    // DecoOrange,
    // /// RGB (215,215,175)
    // PaleGoldenrod,
    // /// RGB (215,215,215)
    // AltoBeige,
    // /// RGB (215,215,255)
    // FogPink,
    // /// RGB (215,255,0)
    // ChartreuseYellow,
    // /// RGB (215,255,95)
    // Canary,
    // /// RGB (215,255,135)
    // Honeysuckle,
    // /// RGB (215,255,175)
    // ReefPaleYellow,
    // /// RGB (215,255,215)
    // SnowyMint,
    // /// RGB (215,255,255)
    // OysterBay,
    // /// RGB (255,0,0)196
    // AnotherRed,
    // /// RGB (255,0,95)
    // DarkRose,
    // /// RGB (255,0,135)98
    // Rose,
    // /// RGB (255,0,175)
    // LightHollywoodCerise,
    // /// RGB (255,0,215)
    // PurplePizzazz,
    // /// RGB (255,0,255)
    // Fuchsia,
    // /// RGB (255,95,0)
    // BlazeOrange,
    // /// RGB (255,95,95)
    // BittersweetOrange,
    // /// RGB (255,95,135)
    // WildWatermelon,
    // /// RGB (255,95,175)
    // DarkHotPink,
    // /// RGB (255,95,215)
    // HotPink,
    // /// RGB (255,95,255)
    // PinkFlamingo,
    // /// RGB (255,135,0)
    // FlushOrange,
    // /// RGB (255,135,95)
    // Salmon,
    // /// RGB (255,135,135)
    // VividTangerine,
    // /// RGB (255,135,175)
    // PinkSalmon,
    // /// RGB (255,135,215)
    // DarkLavenderRose,
    // /// RGB (255,135,255)
    // BlushPink,
    // /// RGB (255,175,0)
    // YellowSea,
    // /// RGB (255,175,95)
    // TexasRose,
    // /// RGB (255,175,135)6
    // Tacao,
    // /// RGB (255,175,175)
    // Sundown,
    // /// RGB (255,175,215)
    // CottonCandy,
    // /// RGB (255,175,255)
    // LavenderRose,
    // /// RGB (255,215,0)20
    // Gold,
    // /// RGB (255,215,95)
    // Dandelion,
    // /// RGB (255,215,135)
    // GrandisCaramel,
    // /// RGB (255,215,175)
    // Caramel,
    // /// RGB (255,215,215)
    // CosmosSalmon,
    // /// RGB (255,215,255)
    // PinkLace,
    // /// RGB (255,255,0)
    // AnotherYellow,
    // /// RGB (255,255,95)
    // LaserLemon,
    // /// RGB (255,255,135)
    // DollyYellow,
    // /// RGB (255,255,175)
    // PortafinoYellow,
    // /// RGB (255,255,215)
    // Cumulus,
    // /// RGB (255,255,255)1
    // AnotherWhite,
    // /// RGB (8,8,8)
    // DarkCodGray,
    // /// RGB (18,18,18)
    // CodGray,
    // /// RGB (28,28,28)
    // LightCodGray,
    // /// RGB (38,38,38)
    // DarkMineShaft,
    // /// RGB (48,48,48)
    // MineShaft,
    // /// RGB (58,58,58)
    // LightMineShaft,
    // /// RGB (68,68,68)
    // DarkTundora,
    // /// RGB (78,78,78)
    // Tundora,
    // /// RGB (88,88,88)
    // ScorpionGray,
    // /// RGB (98,98,98)
    // DarkDoveGray,
    // /// RGB (108,108,108)
    // DoveGray,
    // /// RGB (118,118,118)
    // Boulder,
    // /// RGB (128,128,128)
    // Gray,
    // /// RGB (138,138,138)
    // LightGray,
    // /// RGB (148,148,148)
    // DustyGray,
    // /// RGB (158,158,158)
    // NobelGray,
    // /// RGB (168,168,168)
    // DarkSilverChalice,
    // /// RGB (178,178,178)
    // LightSilverChalice,
    // /// RGB (188,188,188)
    // DarkSilver,
    // /// RGB (198,198,198)
    // Silver,
    // /// RGB (208,208,208)
    // DarkAlto,
    // /// RGB (218,218,218)53
    // Alto,
    // /// RGB (228,228,228)
    // Mercury,
    // /// RGB (238,238,238)
    // GalleryGray,
    #[display("Color::Index({_0})")]
    Index(u8),
    #[display("Color::Rgb({_0}, {_1}, {_2})")]
    Rgb(u8, u8, u8),
    #[default]
    None,
}
const impl Color {
    #[inline]
    #[must_use]
    pub fn intersection(self, other: Self) -> Self {
        match (self, other) {
            (Color::None, _) | (_, Color::None) => Color::None,
            (a, b) if a == b => a,
            _ => Color::None,
        }
    }

    #[inline]
    #[must_use]
    pub fn difference(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Color::None, _) => Color::None,
            (x, Color::None) => x,
            (_a, _b) => Color::None,
        }
    }
    #[inline]
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        match (self, other) {
            (x, Color::None) | (Color::None, x) => x,
            (_, x) => x,
        }
    }

    // pub fn convert(self, target: ColorSpace) -> Color {
    //     match target {
    //         ColorSpace::Ansi => Basic::from(self).into(),
    //         ColorSpace::Index => Indexed::from(self).into(),
    //         ColorSpace::Rgb => Rgb::from(self).into(),
    //     }
    // }
    //
    // pub fn clamp(self, max: ColorSpace) -> Color {
    //     if let Some(source) = self.color_space() {
    //         return match max {
    //             ColorSpace::Ansi => match source {
    //                 ColorSpace::Rgb => self.convert(ColorSpace::Ansi),
    //                 ColorSpace::Index => self.convert(ColorSpace::Ansi),
    //                 _ => self,
    //             },
    //             ColorSpace::Index => match source {
    //                 ColorSpace::Rgb => self.convert(ColorSpace::Index),
    //                 _ => self,
    //             },
    //             ColorSpace::Rgb => self,
    //         };
    //     }
    //
    //     self
    // }

    pub fn color_space(&self) -> Option<ColorSpace> {
        match self {
            Color::None => None,
            Color::Index(_) => Some(ColorSpace::Ansi),
            Color::Rgb(_, _, _) => Some(ColorSpace::Rgb),
            _ => None,
        }
    }
}
const impl From<u32> for Color {
    fn from(value: u32) -> Self {
        Color::Rgb(
            ((value >> 16) & 0xFF) as u8,
            ((value >> 8) & 0xFF) as u8,
            (value & 0xFF) as u8,
        )
    }
}

const impl From<(u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8)) -> Self {
        Color::Rgb(value.0, value.1, value.2)
    }
}

const impl From<u8> for Color {
    fn from(value: u8) -> Self {
        match value {
            0 => Color::Black,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            4 => Color::Blue,
            5 => Color::Magenta,
            6 => Color::Cyan,
            7 => Color::White,
            8 => Color::BrightBlack,
            9 => Color::BrightRed,
            10 => Color::BrightGreen,
            11 => Color::BrightYellow,
            12 => Color::BrightBlue,
            13 => Color::BrightMagenta,
            14 => Color::BrightCyan,
            15 => Color::BrightWhite,
            16..=255 => Color::Index(value),
        }
    }
}

impl Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

const impl BitAnd for Color {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

const impl BitAndAssign for Color {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

const impl BitOr for Color {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

const impl BitOrAssign for Color {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

const impl Sub for Color {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

const impl SubAssign for Color {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersection() {
        for (lhs, rhs, expected) in [
            (Color::None, Color::Black, Color::None),
            (Color::None, Color::None, Color::None),
            (Color::Black, Color::None, Color::None),
            (Color::Black, Color::Black, Color::Black),
        ] {
            assert_eq!(
                lhs.intersection(rhs),
                expected,
                "{:?}.intersection({:?})",
                lhs,
                rhs
            );
        }
    }

    #[test]
    fn test_union() {
        for (lhs, rhs, expected) in [
            (Color::None, Color::None, Color::None),
            (Color::None, Color::Black, Color::Black),
            (Color::Black, Color::None, Color::Black),
            (Color::Black, Color::Black, Color::Black),
        ] {
            assert_eq!(lhs.union(rhs), expected, "{:?}.union({:?})", lhs, rhs);
        }
    }

    #[test]
    fn test_difference() {
        for (lhs, rhs, expected) in [
            (Color::None, Color::None, Color::None),
            (Color::None, Color::Black, Color::None),
            (Color::Black, Color::None, Color::Black),
            (Color::Black, Color::Blue, Color::None),
        ] {
            assert_eq!(
                lhs.difference(rhs),
                expected,
                "{:?}.difference({:?})",
                lhs,
                rhs
            );
        }
    }
}
