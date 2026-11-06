//! Provides shape property interpolation animations.

use std::fmt::{Debug, Formatter};
use std::mem;
use std::sync::Arc;
use crate::animations::{AffectedShapes, Animation, DynEasingFn, DynamicShape, EasingFn, IntoAffectedShapes};
use runim_derive::Animation;
use crate::dispatch::IntoAnimations;
use crate::IntoDuration;

pub use interpolate::Interpolate;
pub use property::Property;

mod interpolate;
pub mod property;

type Part = Box<dyn Fn(&mut DynamicShape, f64) + Send + Sync + 'static>;

pub struct InterpolationBuilder {
    duration: f64,
    shapes: AffectedShapes,
    parts: Vec<Part>
}

impl InterpolationBuilder {
    fn push<T>(&mut self, prop: Property<T>, from: T, to: T, easing: DynEasingFn)
    where T: Interpolate + Send + Sync {
        self.parts.push(Box::new(move |shape, d| {
            let d = easing.apply(d);
            let val = from.lerp(&to, d);
            //drop(mem::replace((prop.get_mut)(shape), val));
            (prop.set)(shape, val);
        }));
    }

    #[must_use = "Interpolation will not be created util build() is called"]
    pub fn linear<T>(mut self, prop: Property<T>, from: T, to: T) -> Self
    where T: Interpolate + Send + Sync {
        self.push(prop, from, to, DynEasingFn::Linear);
        self
    }

    #[must_use = "Interpolation will not be created util build() is called"]
    pub fn with_easing<T>(mut self, prop: Property<T>, from: T, to: T, easing: &'static dyn EasingFn) -> Self
    where T: Interpolate + Send + Sync {
        self.push(prop, from, to, DynEasingFn::Ref(easing));
        self
    }

    #[must_use = "Interpolation will not be created util build() is called"]
    pub fn with_owned_easing<T>(mut self, prop: Property<T>, from: T, to: T, easing: Box<dyn EasingFn>) -> Self
    where T: Interpolate + Send + Sync {
        self.push(prop, from, to, DynEasingFn::Boxed(easing));
        self
    }

    #[must_use = "Animation must be passed to a dispatcher to be played"]
    pub fn build(self) -> Interpolation {
        Interpolation {
            shapes: self.shapes,
            duration: self.duration,
            parts: self.parts,
        }
    }
}

#[derive(Animation)]
pub struct Interpolation {
    shapes: AffectedShapes,
    duration: f64,
    parts: Vec<Part>
}

impl Interpolation {
    pub fn builder<S, D>(shapes: S, duration: D) -> InterpolationBuilder
    where S: IntoAffectedShapes, D: IntoDuration {
        InterpolationBuilder {
            shapes: shapes.into_affected_shape(),
            duration: duration.as_secs_f64(),
            parts: Vec::new(),
        }
    }

    /*#[doc(hidden)]
    pub fn ______new_raw<S, D>(shapes: S, duration: D, parts: Vec<Part>) -> InterpolationBuilder
    where S: IntoAffectedShapes, D: IntoDuration {
        InterpolationBuilder {
            shapes: shapes.into_affected_shape(),
            duration: duration.as_secs_f64(),
            parts
        }
    }*/
}

impl Animation for Interpolation {
    fn apply(&self, state: &mut DynamicShape, d: f64) {
        for x in self.parts.iter() {
            x(state, d);
        }
    }
}

impl Debug for Interpolation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Interpolation{ shapes: ")?;
        Debug::fmt(&self.shapes, f)?;
        f.write_str(", duration: ")?;
        Debug::fmt(&self.duration, f)?;
        f.write_str(" }")
    }
}

impl IntoAnimations for InterpolationBuilder {
    fn into_animations(self) -> Vec<Arc<dyn Animation>> {
        vec![Arc::new(self.build())]
    }
}
