# Runim - Rust Animation Library
Runim (***Ru***st A***nim***ation Library) is a library providing tools for creating and rendering
animations using Rust programming language inspired by [Manim](https://www.manim.community).

## Design
All animations created with library are defined using Rust code:
```rust,ignore
dispatch.play(
    interpolate!(&shape, 6.0,
        property::POSITION: Vec2::new(-2.0, 0.0) => Vec2::new(2.0, 0.0); &easing::EASE_IN_OUT,
        property::COLOR: Color::WHITE => Color::RED; LINEAR,
    )
)?;
dispatch.play(
    shape.move_to(3.0, Vec2::new(2.0, 0.0)
)?;
```
All things that can be rendered called [Shapes](./shapes.md).
And all things that happen to them are called [Animations](./animations.md).
