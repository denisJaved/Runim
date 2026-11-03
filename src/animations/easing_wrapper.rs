use std::fmt::{Debug, Formatter, Write};
use crate::animations::{AffectedShapes, Animation, AnimationBase, DynamicShape};

/// Wraps another animation to provide custom interpolation.
pub struct AnimationEasing {
    parent: Box<dyn Animation>,
    fnc: DynFn
}

impl AnimationEasing {
    pub fn new<A: Animation>(
        parent: A,
        fnc: Box<dyn EasingFn>
    ) -> AnimationEasing {
        AnimationEasing {
            parent: Box::new(parent),
            fnc: DynFn::Boxed(fnc),
        }
    }
    
    pub fn new_ref<A: Animation>(
        parent: A,
        fnc: &'static dyn EasingFn
    ) -> AnimationEasing {
        AnimationEasing {
            parent: Box::new(parent),
            fnc: DynFn::Ref(fnc),
        }
    }
}

impl Debug for AnimationEasing {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Interpolated[")?;
        self.parent.fmt(f)?;
        f.write_char(']')
    }
}

impl AnimationBase for AnimationEasing {
    fn duration(&self) -> f64 {
        self.parent.duration()
    }

    fn affected_shapes(&self) -> &AffectedShapes {
        self.parent.affected_shapes()
    }
}

impl Animation for AnimationEasing {
    fn apply(&self, state: &mut DynamicShape, d: f64) {
        let d = match &self.fnc {
            DynFn::Ref(r) => r.apply(d),
            DynFn::Boxed(b) => b.apply(d)
        };
        self.parent.apply(state, d);
    }
}

enum DynFn {
    Ref(&'static dyn EasingFn),
    Boxed(Box<dyn EasingFn>)
}

pub trait EasingFn: Send + Sync {
    fn apply(&self, v: f64) -> f64;
}

impl<F: Fn(f64) -> f64 + Send + Sync> EasingFn for F {
    fn apply(&self, v: f64) -> f64 {
        self(v)
    }
}
