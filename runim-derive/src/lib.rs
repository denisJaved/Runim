use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Shape, attributes(shape))]
pub fn derive_shape_base(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let mut has_color = false;

    for attr in &input.attrs {
        if attr.path().is_ident("shape") {
            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("colored") {
                    has_color = true;
                    Ok(())
                } else {
                    Err(meta.error("unknown shape option"))
                }
            }) {
                return err.to_compile_error().into();
            }
        }
    }

    let as_colored;
    let colored;

    if has_color {
        as_colored = quote! {
            fn as_colored(&self) -> Option<&dyn runim::shapes::ColoredShape> {
                Some(self)
            }

            fn as_colored_mut(&mut self) -> Option<&mut dyn runim::shapes::ColoredShape> {
                Some(self)
            }
        };
        colored = quote! {
            impl runim::shapes::ColoredShape for #name {
                fn color(&self) -> &runim::shapes::Color {
                    &self.color
                }

                fn color_mut(&mut self) -> &mut runim::shapes::Color {
                    &mut self.color
                }

                fn set_color(&mut self, color: runim::shapes::Color) {
                    self.color = color;
                }
            }
        }
    } else {
        as_colored = quote! {
            fn as_colored(&self) -> Option<&dyn runim::shapes::ColoredShape> {
                None
            }

            fn as_colored_mut(&mut self) -> Option<&mut dyn runim::shapes::ColoredShape> {
                None
            }
        };
        colored = quote! { }
    };

    let expanded = quote! {
        impl runim::shapes::ShapeBase for #name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
            fn clone_raw(&self) -> Box<dyn runim::shapes::Shape> {
                Box::new(core::clone::Clone::clone(self))
            }
            #as_colored
        }
        #colored
    };

    expanded.into()
}


#[proc_macro_derive(Animation, attributes(animation))]
pub fn derive_animation_base(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut has_compositor = false;

    for attr in &input.attrs {
        if attr.path().is_ident("animation") {
            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("custom_compositor") {
                    has_compositor = true;
                    Ok(())
                } else {
                    Err(meta.error("unknown animation option"))
                }
            }) {
                return err.to_compile_error().into();
            }
        }
    }

    let compositor = if has_compositor {
        quote! {}
    } else {
        quote! {
            impl runim::animations::AnimationCompositor for #name {}
        }
    };

    let expanded = quote! {
        impl runim::animations::AnimationBase for #name {
            fn duration(&self) -> f64 {
                self.duration
            }

            fn affected_shapes(&self) -> &runim::animations::AffectedShapes {
                &self.shapes
            }
        }
        #compositor
    };

    expanded.into()
}
