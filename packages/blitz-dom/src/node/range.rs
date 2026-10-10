use super::{ElementData, Node, SpecialElementData};
use crate::local_name;

/// The range control's current value, independent of its default attribute.
#[derive(Clone)]
pub struct RangeInput {
    pub value: f64,
    pub dirty: bool,
}

/// One geometry contract for native painting and pointer defaults (CSS pixels).
pub const RANGE_THUMB_SIZE: f64 = 16.0;

pub struct RangeConstraints {
    pub min: f64,
    pub max: f64,
    pub step: Option<f64>,
    base: f64,
    precision: i32,
}

fn number(value: &str) -> Option<f64> {
    crate::document::valid_number_value(value).then(|| value.parse().unwrap())
}

fn precision(value: &str) -> i32 {
    let (mantissa, exponent) = value.split_once(['e', 'E']).unwrap_or((value, "0"));
    let fractional = mantissa
        .split_once('.')
        .map_or(0, |(_, tail)| tail.len() as i32);
    (fractional - exponent.parse::<i32>().unwrap_or(0)).clamp(0, 308)
}

impl RangeConstraints {
    pub fn sanitize(&self, value: Option<f64>) -> f64 {
        let value = value
            .unwrap_or(self.min / 2.0 + self.max / 2.0)
            .clamp(self.min, self.max);
        let Some(step) = self.step else { return value };
        let count = (value - self.base) / step;
        if !count.is_finite() {
            return value;
        }
        // HTML chooses the greater allowed value when equally near two steps.
        let tolerance = f64::EPSILON * count.abs().max(1.0) * 8.0;
        let count = (count + 0.5 + tolerance)
            .floor()
            .max(((self.min - self.base) / step - tolerance).ceil())
            .min(((self.max - self.base) / step + tolerance).floor());
        let value = (self.base + count * step).clamp(self.min, self.max);
        let scale = 10_f64.powi(self.precision);
        if (value * scale).is_finite() {
            (value * scale).round() / scale
        } else {
            value
        }
    }

    pub fn parse(&self, value: &str) -> f64 {
        self.sanitize(number(value))
    }
}

impl ElementData {
    pub fn range_constraints(&self) -> Option<RangeConstraints> {
        if self.name.local != local_name!("input")
            || !self
                .attr(local_name!("type"))
                .is_some_and(|kind| kind.eq_ignore_ascii_case("range"))
        {
            return None;
        }
        let min_text = self.attr(local_name!("min")).unwrap_or("");
        let max_text = self.attr(local_name!("max")).unwrap_or("");
        let value_text = self.attr(local_name!("value")).unwrap_or("");
        let step_text = self.attr(local_name!("step")).unwrap_or("");
        let min = number(min_text).unwrap_or(0.0);
        Some(RangeConstraints {
            min,
            max: number(max_text).unwrap_or(100.0).max(min),
            step: if step_text.eq_ignore_ascii_case("any") {
                None
            } else {
                Some(
                    number(step_text)
                        .filter(|value| *value > 0.0)
                        .unwrap_or(1.0),
                )
            },
            base: number(min_text)
                .or_else(|| number(value_text))
                .unwrap_or(0.0),
            precision: precision(min_text)
                .max(precision(value_text))
                .max(precision(step_text)),
        })
    }

    pub fn range_value(&self) -> Option<f64> {
        let constraints = self.range_constraints()?;
        Some(match &self.special_data {
            SpecialElementData::RangeInput(input) => input.value,
            _ => constraints.parse(self.attr(local_name!("value")).unwrap_or("")),
        })
    }

    pub(crate) fn sync_range_attribute(&mut self, attribute: &str) {
        let Some(constraints) = self.range_constraints() else {
            return;
        };
        let (current, dirty) = match &self.special_data {
            SpecialElementData::RangeInput(input) => (Some(input.value), input.dirty),
            SpecialElementData::TextInput(input) => {
                (number(&input.editor.text().to_string()), input.dirty_value)
            }
            _ => (None, false),
        };
        let value = if current.is_none() || (!dirty && attribute == "value") {
            constraints.parse(self.attr(local_name!("value")).unwrap_or(""))
        } else {
            constraints.sanitize(current)
        };
        self.special_data = SpecialElementData::RangeInput(RangeInput { value, dirty });
    }

    pub fn set_range_value(&mut self, value: &str) {
        let Some(constraints) = self.range_constraints() else {
            return;
        };
        self.special_data = SpecialElementData::RangeInput(RangeInput {
            value: constraints.parse(value),
            dirty: true,
        });
    }
}

impl Node {
    pub fn range_axis(&self) -> (bool, bool) {
        self.primary_styles().map_or((false, false), |style| {
            let wm = style.writing_mode;
            (wm.is_vertical(), !wm.is_bidi_ltr())
        })
    }
}
