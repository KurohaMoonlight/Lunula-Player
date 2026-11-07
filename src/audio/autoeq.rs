use crate::models::{FilterSlope, FilterType, ParametricBand, ParametricEqPreset, PreampMode};

/// Parses an AutoEq ParametricEQ.txt or EqualizerAPO formatted string into a `ParametricEqPreset`.
///
/// Supported formats:
/// 1. `Preamp: -X.X dB`
/// 2. `Filter N: ON/OFF PK/PEAK/LS/LSC/LOW_SHELF/HS/HSC/HIGH_SHELF/HP/HPF/LP/LPF/NOTCH Fc <freq> Hz Gain <gain> dB Q <q>`
pub fn parse_autoeq_text(preset_name: &str, content: &str) -> Result<ParametricEqPreset, String> {
    let mut manual_preamp_db = 0.0f32;
    let mut has_preamp = false;
    let mut bands = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Check for Preamp
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("preamp:") {
            let rest = trimmed["preamp:".len()..].trim();
            let num_str = rest.split_whitespace().next().unwrap_or(rest).trim_end_matches("dB").trim_end_matches("db").trim();
            if let Ok(val) = num_str.parse::<f32>() {
                manual_preamp_db = val;
                has_preamp = true;
            }
            continue;
        }

        // Check for Filter line: e.g. "Filter 1: ON PK Fc 28 Hz Gain 7.2 dB Q 0.70"
        if lower.starts_with("filter") {
            if let Some(colon_idx) = trimmed.find(':') {
                let spec = trimmed[colon_idx + 1..].trim();
                if let Some(band) = parse_filter_spec(spec) {
                    bands.push(band);
                }
            }
        }
    }

    if bands.is_empty() {
        return Err("No valid filter bands found in AutoEq configuration".to_string());
    }

    Ok(ParametricEqPreset {
        name: preset_name.to_string(),
        bands,
        preamp_mode: if has_preamp { PreampMode::Manual } else { PreampMode::Auto },
        manual_preamp_db,
    })
}

fn parse_filter_spec(spec: &str) -> Option<ParametricBand> {
    let tokens: Vec<&str> = spec.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let mut enabled = true;
    let mut filter_type = FilterType::Peaking;
    let mut freq = 1000.0f32;
    let mut gain_db = 0.0f32;
    let mut q = 1.414f32;

    let mut i = 0;
    while i < tokens.len() {
        let t = tokens[i].to_ascii_uppercase();
        match t.as_str() {
            "ON" => {
                enabled = true;
                i += 1;
            }
            "OFF" => {
                enabled = false;
                i += 1;
            }
            "PK" | "PEAK" | "PEAKING" => {
                filter_type = FilterType::Peaking;
                i += 1;
            }
            "LS" | "LSC" | "LOW_SHELF" | "LOWSHELF" => {
                filter_type = FilterType::LowShelf;
                i += 1;
            }
            "HS" | "HSC" | "HIGH_SHELF" | "HIGHSHELF" => {
                filter_type = FilterType::HighShelf;
                i += 1;
            }
            "HP" | "HPF" | "HIGH_PASS" | "HIGHPASS" => {
                filter_type = FilterType::HighPass;
                i += 1;
            }
            "LP" | "LPF" | "LOW_PASS" | "LOWPASS" => {
                filter_type = FilterType::LowPass;
                i += 1;
            }
            "NO" | "NOTCH" => {
                filter_type = FilterType::Notch;
                i += 1;
            }
            "FC" => {
                if i + 1 < tokens.len() {
                    let val_str = tokens[i + 1].trim_end_matches("Hz").trim_end_matches("hz");
                    if let Ok(v) = val_str.parse::<f32>() {
                        freq = v.clamp(20.0, 20000.0);
                    }
                    i += 2;
                    // Skip optional "Hz" token if separate
                    if i < tokens.len() && tokens[i].eq_ignore_ascii_case("Hz") {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
            "GAIN" => {
                if i + 1 < tokens.len() {
                    let val_str = tokens[i + 1].trim_end_matches("dB").trim_end_matches("db");
                    if let Ok(v) = val_str.parse::<f32>() {
                        gain_db = v.clamp(-24.0, 24.0);
                    }
                    i += 2;
                    // Skip optional "dB" token if separate
                    if i < tokens.len() && tokens[i].eq_ignore_ascii_case("dB") {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
            "Q" => {
                if i + 1 < tokens.len() {
                    if let Ok(v) = tokens[i + 1].parse::<f32>() {
                        q = v.clamp(0.1, 10.0);
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    Some(ParametricBand {
        enabled,
        filter_type,
        freq,
        gain_db,
        q,
        slope: FilterSlope::Db12,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_autoeq_standard_sennheiser() {
        let sample = r#"
# AutoEq profile for Sennheiser HD 600
Preamp: -6.5 dB
Filter 1: ON PK Fc 28 Hz Gain 7.2 dB Q 0.70
Filter 2: ON LSC Fc 105 Hz Gain 5.5 dB Q 0.71
Filter 3: ON PK Fc 200 Hz Gain -1.8 dB Q 1.40
Filter 4: ON PK Fc 1250 Hz Gain -2.1 dB Q 2.00
Filter 5: ON PK Fc 3000 Hz Gain 4.8 dB Q 1.50
Filter 6: ON HSC Fc 10000 Hz Gain -2.0 dB Q 0.70
"#;
        let preset = parse_autoeq_text("Sennheiser HD 600 (Harman)", sample).expect("Failed to parse");
        assert_eq!(preset.name, "Sennheiser HD 600 (Harman)");
        assert_eq!(preset.preamp_mode, PreampMode::Manual);
        assert!((preset.manual_preamp_db - (-6.5)).abs() < 1e-4);
        assert_eq!(preset.bands.len(), 6);

        // Band 1
        assert_eq!(preset.bands[0].filter_type, FilterType::Peaking);
        assert!((preset.bands[0].freq - 28.0).abs() < 1e-3);
        assert!((preset.bands[0].gain_db - 7.2).abs() < 1e-3);
        assert!((preset.bands[0].q - 0.70).abs() < 1e-3);

        // Band 2 (Low Shelf)
        assert_eq!(preset.bands[1].filter_type, FilterType::LowShelf);
        assert!((preset.bands[1].freq - 105.0).abs() < 1e-3);
        assert!((preset.bands[1].gain_db - 5.5).abs() < 1e-3);

        // Band 6 (High Shelf)
        assert_eq!(preset.bands[5].filter_type, FilterType::HighShelf);
        assert!((preset.bands[5].freq - 10000.0).abs() < 1e-3);
        assert!((preset.bands[5].gain_db - (-2.0)).abs() < 1e-3);
    }
}

