use std::fs;
use std::io::{self, Write};

use bootspec::{BootJson, SpecialisationName};

use crate::{Generation, Result};

mod efi;
mod toplevel;

pub use efi::EfiProgram;
pub use toplevel::BootableToplevel;

pub enum Bootable {
    Linux(BootableToplevel),
    Efi(EfiProgram),
}

/// `flatten` takes in a list of [`Generation`]s and returns a list of [`BootableToplevel`]s by:
///
/// 1. transforming each [`Generation`] into a [`BootableToplevel`]; and
/// 2. recursing into each [`Generation`]s specialisations (if any) and transforming them into
///    [`BootableToplevel`]s of their own (and so on and so forth).
///
/// This makes it easy to create boot entries for all possible [`BootableToplevel`]s (both the
/// "system profile" as well as its many possible specialisations), while also ensuring we encounter
/// potential infinite recursion as early as possible.
pub fn flatten(inputs: Vec<Generation>) -> Result<Vec<BootableToplevel>> {
    self::flatten_impl(inputs, None)
}

fn flatten_impl(
    inputs: Vec<Generation>,
    specialisation_name: Option<SpecialisationName>,
) -> Result<Vec<BootableToplevel>> {
    let mut toplevels = Vec::new();

    for input in inputs {
        let toplevel = input.bootspec.toplevel.clone();

        toplevels.push(BootableToplevel {
            system_version: input.bootspec.system_version,
            kernel: input.bootspec.kernel,
            kernel_version: input.bootspec.kernel_version,
            kernel_params: input.bootspec.kernel_params,
            init: input.bootspec.init,
            initrd: input.bootspec.initrd,
            toplevel,
            specialisation_name: specialisation_name.clone(),
            generation_index: input.index,
            profile_name: input.profile.clone(),
        });

        for (name, desc) in input.bootspec.specialisation {
            let bootspec_path = if let Some(bootspec_path) = desc.bootspec {
                bootspec_path.0
            } else {
                return Err(format!("Specialisation '{}' didn't have a bootspec", name.0).into());
            };

            writeln!(
                io::stderr(),
                "Flattening specialisation '{name}' of toplevel {toplevel}: {path}",
                toplevel = input.bootspec.toplevel.0.display(),
                name = name.0,
                path = bootspec_path.display()
            )?;

            let json = fs::read_to_string(&bootspec_path)?;
            let parsed: BootJson = serde_json::from_str(&json)?;
            let gen = Generation {
                index: input.index,
                profile: input.profile.clone(),
                bootspec: parsed,
            };

            toplevels.extend(self::flatten_impl(vec![gen], Some(name))?);
        }
    }

    Ok(toplevels)
}
