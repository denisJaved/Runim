use proc_macro::TokenStream;
use quote::quote;
use syn::__private::TokenStream2;
use syn::parse::{Parse, ParseStream};
use syn::{DeriveInput, Expr, Path, Token, parse_macro_input};
// TODO: Refactor this mess

// ==================== interpolate! ====================
// TODO: Test interpolate!. Partically vibecoded
struct IplInput {
    shape: Expr,
    duration: Expr,
    items: Vec<IplItem>
}
impl Parse for IplInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let shape: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let duration: Expr = input.parse()?;

        let mut items = Vec::new();
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() { break; }
            items.push(input.parse::<IplItem>()?)
        }

        Ok(IplInput { shape, duration, items })
    }
}
struct IplItem {
    property: Path,
    from: Expr,
    to: Expr,
    easing: EasingKind
}
impl Parse for IplItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let property: Path = input.parse()?;
        input.parse::<Token![:]>()?;
        let from: Expr = input.parse()?;
        input.parse::<Token![=>]>()?;
        let to: Expr = input.parse()?;
        input.parse::<Token![;]>()?;
        let easing = parse_easing(input)?;
        Ok(IplItem { property, from, to, easing })
    }
}
fn parse_easing(input: ParseStream) -> syn::Result<EasingKind> {
    let expr: Expr = input.parse()?;
    match &expr {
        Expr::Path(p) if p.path.is_ident("LINEAR") => Ok(EasingKind::Linear),
        Expr::Reference(_) => Ok(EasingKind::Ref(expr)),
        _ => Ok(EasingKind::Owned(expr)),
    }
}
enum EasingKind {
    Linear,
    Ref(Expr),
    Owned(Expr)
}

fn expand(input: IplInput) -> TokenStream2 { // Maybe I shouldn't use __private::TokenStream2
    let IplInput { shape, duration, items } = input;

    let calls = items.into_iter().map(|IplItem { property, from, to, easing }| {
        match easing {
            EasingKind::Linear => quote! {
                .linear(#property, #from, #to)
            },
            EasingKind::Ref(e) => quote! {
                .with_easing(#property, #from, #to, #e)
            },
            EasingKind::Owned(e) => quote! {
                .with_owned_easing(#property, #from, #to, #e)
            },
        }
    });

    quote! {
        ::runim::prelude::Interpolation::builder(#shape, #duration)
            #(#calls)*
            .build()
    }
}

#[proc_macro]
pub fn interpolate(input: TokenStream) -> TokenStream {
    expand(parse_macro_input!(input as IplInput)).into()
}


// ==================== derive macros ====================
#[proc_macro_derive(Shape, attributes(shape))]
pub fn derive_shape_base(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let mut has_color = false;
    let mut no_cache = false;

    for attr in &input.attrs {
        if attr.path().is_ident("shape") {
            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("colored") {
                    has_color = true;
                    Ok(())
                } else if meta.path.is_ident("no_cache") {
                    no_cache = true;
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
            fn as_colored(&self) -> Option<&dyn ::runim::shapes::ColoredShape> {
                Some(self)
            }

            fn as_colored_mut(&mut self) -> Option<&mut dyn ::runim::shapes::ColoredShape> {
                Some(self)
            }
        };
        colored = quote! {
            impl ::runim::shapes::ColoredShape for #name {
                fn color(&self) -> &::runim::shapes::Color {
                    &self.color
                }

                fn color_mut(&mut self) -> &mut ::runim::shapes::Color {
                    &mut self.color
                }

                fn set_color(&mut self, color: ::runim::shapes::Color) {
                    self.color = color;
                }
            }
        }
    } else {
        as_colored = quote! {
            fn as_colored(&self) -> Option<&dyn ::runim::shapes::ColoredShape> {
                None
            }

            fn as_colored_mut(&mut self) -> Option<&mut dyn ::runim::shapes::ColoredShape> {
                None
            }
        };
        colored = quote! { }
    };

    let expanded = quote! {
        impl ::runim::shapes::ShapeBase for #name {
            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
                self
            }
            fn clone_raw(&self) -> Box<dyn ::runim::shapes::Shape> {
                Box::new(::core::clone::Clone::clone(self))
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
            impl ::runim::animations::AnimationCompositor for #name {}
        }
    };

    let expanded = quote! {
        impl ::runim::animations::AnimationBase for #name {
            fn duration(&self) -> f64 {
                self.duration
            }

            fn affected_shapes(&self) -> &::runim::animations::AffectedShapes {
                &self.shapes
            }
        }
        #compositor
    };

    expanded.into()
}


#[proc_macro_derive(AsAny)]
pub fn derive_as_any(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl ::runim::util_traits::AsAny for #name {
            fn as_any(&self) -> &dyn ::core::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn ::core::any::Any {
                self
            }
        }
    };

    expanded.into()
}
