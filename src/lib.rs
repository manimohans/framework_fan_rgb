use framework_lib::chromium_ec::commands::RgbS;
use framework_lib::chromium_ec::{CrosEc, CrosEcDriverType, EcError};

/// Convert a raw 24-bit RGB value into the EC payload struct.
pub fn rgb_from_u32(value: u32) -> RgbS {
    RgbS {
        r: ((value & 0x00FF_0000) >> 16) as u8,
        g: ((value & 0x0000_FF00) >> 8) as u8,
        b: (value & 0x0000_00FF) as u8,
    }
}

/// Parse a textual color representation into a 24-bit RGB value accepted by the EC.
///
/// Supports `0xRRGGBB`, `#RRGGBB`, or decimal literals (0-16777215).
pub fn parse_color(input: &str) -> Result<RgbS, String> {
    let trimmed = input.trim();
    let (radix, digits) = if let Some(hex) = trimmed.strip_prefix("0x") {
        (16, hex)
    } else if let Some(hex) = trimmed.strip_prefix("0X") {
        (16, hex)
    } else if let Some(hex) = trimmed.strip_prefix('#') {
        (16, hex)
    } else {
        match u32::from_str_radix(trimmed, 10) {
            Ok(value) => return rgb_from_value(trimmed, value),
            Err(_) => (16, trimmed),
        }
    };

    let value = u32::from_str_radix(digits, radix)
        .map_err(|err| format!("invalid color `{input}`: {err}"))?;

    rgb_from_value(input, value)
}

fn rgb_from_value(input: &str, value: u32) -> Result<RgbS, String> {
    if value > 0x00FF_FFFF {
        return Err(format!(
            "color `{input}` exceeds 24-bit RGB range (0x000000..=0xFFFFFF)"
        ));
    }
    Ok(rgb_from_u32(value))
}

/// Apply RGB colors starting at a given key index using the Framework EC.
pub fn apply_colors(
    start_key: u8,
    colors: Vec<RgbS>,
    driver: Option<CrosEcDriverType>,
) -> Result<(), EcError> {
    let ec = match driver {
        Some(driver) => CrosEc::with(driver).ok_or_else(|| {
            EcError::DeviceError(format!(
                "driver {driver:?} is not available on this platform"
            ))
        })?,
        None => CrosEc::new(),
    };

    ec.rgbkbd_set_color(start_key, colors)
}

/// Convert a `RgbS` color to a hex string (`#RRGGBB`).
pub fn rgb_to_hex_string(color: RgbS) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}
