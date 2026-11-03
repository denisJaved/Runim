use crate::animations::{AffectedShapes, Animation, AnimationCompositor, DynamicShape};
use crate::dispatch::IntoAnimations;
use runim_derive::Animation;
use std::sync::Arc;

#[derive(Animation, Debug, Clone)]
#[animation(custom_compositor)]
pub struct SequentialCompositor {
    shapes: AffectedShapes,
    duration: f64,
    parts: Vec<(f64, Arc<dyn Animation>)>
}

impl SequentialCompositor {
    pub fn new<A>(animations: A) -> SequentialCompositor
    where A: IntoAnimations {
        let mut offset: f64 = 0.0;
        let mut shapes = AffectedShapes::Vector(vec![]);

        let parts: Vec<(f64, Arc<dyn Animation>)> = animations.into_animations()
            .into_iter()
            .map(|r| {
                // Add all affects shapes from all animations
                shapes.append(r.affected_shapes().clone());

                let o = offset;
                offset += r.duration();
                (o, r)
            })
            .collect();

        shapes.dedup();

        SequentialCompositor {
            shapes,
            duration: offset,
            parts,
        }
    }
}

impl Animation for SequentialCompositor {
    fn apply(&self, state: &mut DynamicShape, d: f64) {
        let x = d * self.duration;
        for i in self.parts.iter() {
            if i.0 > x { continue }

            // TODO: Check that x.1 accepts current shape

            let d0 = (x - i.0) / i.1.duration();
            i.1.apply(state, d0);
        }
    }
}

impl AnimationCompositor for SequentialCompositor {
    fn then<A: Animation>(mut self, other: A) -> SequentialCompositor
    where Self: Sized
    {
        self.duration += other.duration();
        self.shapes.append(other.affected_shapes().clone());
        self.shapes.dedup();

        let last = self.parts.last().unwrap();
        let offset = last.0 + last.1.duration();

        self.parts.push((offset, Arc::new(other)));
        self
    }
}

#[derive(Animation, Debug, Clone)]
#[animation(custom_compositor)]
pub struct ParallelCompositor {
    shapes: AffectedShapes,
    duration: f64,
    parts: Vec<Arc<dyn Animation>>
}

impl ParallelCompositor {
    pub fn new<A>(animations: A) -> ParallelCompositor
    where A: IntoAnimations {
        let parts = animations.into_animations();

        let mut duration: f64 = 0.0;
        let mut shapes = AffectedShapes::Vector(vec![]);
        for x in parts.iter() {
            if x.duration() > duration {
                duration = x.duration();
            }
            shapes.append(x.affected_shapes().clone());
        }

        shapes.dedup();

        ParallelCompositor {
            shapes,
            duration,
            parts,
        }
    }
}

impl Animation for ParallelCompositor {
    fn apply(&self, state: &mut DynamicShape, d: f64) {
        let x = d * self.duration;
        for i in self.parts.iter() {
            let d0 = (x / i.duration()).min(1.0);
            i.apply(state, d0);
        }
    }
}

impl AnimationCompositor for ParallelCompositor {
    fn with<A: Animation>(mut self, other: A) -> ParallelCompositor
    where Self: Sized
    {
        if other.duration() > self.duration {
            self.duration = other.duration();
        }
        self.parts.push(Arc::new(other));
        self
    }
}