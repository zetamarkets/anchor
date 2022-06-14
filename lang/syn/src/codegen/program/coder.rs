use crate::codegen::program::common::*;
use crate::Program;
use heck::CamelCase;
use quote::quote;

pub fn generate(program: &Program) -> proc_macro2::TokenStream {
    // Defines all global program instructions in an enum.
    // let mut global_ix_variants = program.ixs.clone();
    // global_ix_variants.sort_by_key(|x| x.anchor_ident.to_string());
    // global_ix_variants.dedup_by_key(|x| x.anchor_ident.to_string());
    // let global_ix_variants: Vec<proc_macro2::TokenStream> = global_ix_variants
    //     .iter()
    //     .map(|ix| {
    //         let name = ix.anchor_ident.to_string();
    //         let ix_name_camel =
    //             proc_macro2::Ident::new(&name.to_camel_case(), ix.anchor_ident.span());
    //         let ix_name_camel_variant: proc_macro2::TokenStream =
    //             format!("{}{}", ix_name_camel, "Variant").parse().unwrap();
    //         quote! {
    //             #ix_name_camel_variant(#ix_name_camel),
    //         }
    //     })
    //     .collect();
    let global_ix_variants: Vec<proc_macro2::TokenStream> = program
        .ixs
        .iter()
        .map(|ix| {
            let name = &ix.raw_method.sig.ident.to_string();
            let ix_name_camel =
                proc_macro2::Ident::new(&name.to_camel_case(), ix.raw_method.sig.ident.span());
            let ix_name_camel_variant: proc_macro2::TokenStream =
                format!("{}{}", ix_name_camel, "Variant").parse().unwrap();
            quote! {
                #ix_name_camel_variant(#ix_name_camel),
            }
        })
        .collect();

    // Decode all global instructions.
    let global_decoder_arms: Vec<proc_macro2::TokenStream> = program
        .ixs
        .iter()
        .map(|ix| {
            let ix_method_name = &ix.raw_method.sig.ident;
            let ix_method_name_camel: proc_macro2::TokenStream =
                ix_method_name.to_string().to_camel_case().parse().unwrap();
            let sighash_arr = sighash(SIGHASH_GLOBAL_NAMESPACE, &ix_method_name.to_string());
            let sighash_tts: proc_macro2::TokenStream =
                format!("{:?}", sighash_arr).parse().unwrap();
            if ix.args.len() > 0 {
                let ix_name_camel_variant: proc_macro2::TokenStream =
                    format!("{}{}", ix_method_name_camel, "Variant").parse().unwrap();
                quote! {
                    #sighash_tts => {
                        let ix_decoded = instruction::#ix_method_name_camel::try_from_slice(ix_data)?;
                        Ok(Some(Self::#ix_name_camel_variant(ix_decoded)))
                    }
                }
            } else {
                quote! {
                    #sighash_tts => {
                        Ok(None)
                    }
                }
            }
        })
        .collect();
    // let fallback_fn = gen_fallback(program).unwrap_or(quote! {
    //     Err(anchor_lang::error::ErrorCode::InstructionFallbackNotFound.into())
    // });
    let fallback_fn = quote! {
        Err(anchor_lang::error::ErrorCode::InstructionFallbackNotFound.into())
    };
    quote! {
        /// Performs method dispatch.
        ///
        /// Each method in an anchor program is uniquely defined by a namespace
        /// and a rust identifier (i.e., the name given to the method). These
        /// two pieces can be combined to creater a method identifier,
        /// specifically, Anchor uses
        ///
        /// Sha256("<namespace>::<rust-identifier>")[..8],
        ///
        /// where the namespace can be one of three types. 1) "global" for a
        /// regular instruction, 2) "state" for a state struct instruction
        /// handler and 3) a trait namespace (used in combination with the
        /// `#[interface]` attribute), which is defined by the trait name, e..
        /// `MyTrait`.
        ///
        /// With this 8 byte identifier, Anchor performs method dispatch,
        /// matching the given 8 byte identifier to the associated method
        /// handler, which leads to user defined code being eventually invoked.
        pub mod coder {
            use super::*;

            #[repr(u8)]
            #[derive(Debug)]
            pub enum AnchorInstruction {
                #(#global_ix_variants)*
            }

            impl AnchorInstruction {
                pub fn decode(data: &[u8]) -> anchor_lang::Result<Option<Self>> {
                    // Split the instruction data into the first 8 byte method
                    // identifier (sighash) and the serialized instruction data.
                    let mut ix_data: &[u8] = data;
                    let sighash: [u8; 8] = {
                        let mut sighash: [u8; 8] = [0; 8];
                        sighash.copy_from_slice(&ix_data[..8]);
                        ix_data = &ix_data[8..];
                        sighash
                    };

                    match sighash {
                        #(#global_decoder_arms)*
                        _ => {
                            #fallback_fn
                        }
                    }
                }
            }
        }

        // pub fn decode(
        //     data: &[u8],
        // ) -> anchor_lang::Result<()> {
        //     // Split the instruction data into the first 8 byte method
        //     // identifier (sighash) and the serialized instruction data.
        //     let mut ix_data: &[u8] = data;
        //     let sighash: [u8; 8] = {
        //         let mut sighash: [u8; 8] = [0; 8];
        //         sighash.copy_from_slice(&ix_data[..8]);
        //         ix_data = &ix_data[8..];
        //         sighash
        //     };

        //     match sighash {
        //         #(#global_decoder_arms)*
        //         _ => {
        //             #fallback_fn
        //         }
        //     }
        // }
    }
}
