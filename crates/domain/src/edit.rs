use serde::{Deserialize, Serialize};

use crate::DomainError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct EditParams {
    pub exposure: f32,
    pub contrast: f32,
    pub temperature: f32,
    pub tint: f32,
    pub highlights: f32,
    pub shadows: f32,
}

impl Default for EditParams {
    fn default() -> Self {
        Self {
            exposure: 0.0,
            contrast: 0.0,
            temperature: 0.0,
            tint: 0.0,
            highlights: 0.0,
            shadows: 0.0,
        }
    }
}

impl EditParams {
    pub fn validate(&self) -> Result<(), DomainError> {
        if !self.exposure.is_finite() {
            return Err(DomainError::NonFiniteEditParam("exposure"));
        }
        if !self.contrast.is_finite() {
            return Err(DomainError::NonFiniteEditParam("contrast"));
        }
        if !self.temperature.is_finite() {
            return Err(DomainError::NonFiniteEditParam("temperature"));
        }
        if !self.tint.is_finite() {
            return Err(DomainError::NonFiniteEditParam("tint"));
        }
        if !self.highlights.is_finite() {
            return Err(DomainError::NonFiniteEditParam("highlights"));
        }
        if !self.shadows.is_finite() {
            return Err(DomainError::NonFiniteEditParam("shadows"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_are_zeroed() {
        let params = EditParams::default();
        assert_eq!(params.exposure, 0.0);
        assert_eq!(params.contrast, 0.0);
        assert_eq!(params.temperature, 0.0);
        assert_eq!(params.tint, 0.0);
        assert_eq!(params.highlights, 0.0);
        assert_eq!(params.shadows, 0.0);
    }

    #[test]
    fn validate_rejects_non_finite_values() {
        let params = EditParams {
            exposure: f32::NAN,
            ..EditParams::default()
        };
        assert!(matches!(
            params.validate(),
            Err(DomainError::NonFiniteEditParam("exposure"))
        ));
    }
}
