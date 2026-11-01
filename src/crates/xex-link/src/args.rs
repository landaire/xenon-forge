use std::path::Path;

use rootcause::IntoReport;

use crate::error::Result;
use crate::error::XexLinkError;
use crate::error::io;
use crate::manifest::Manifest;
use crate::resources::ResourceFile;

pub const USAGE: &str = "\
usage: xex-link <input.elf> <output.xex> [--manifest PATH] [--resource NAME=PATH]...
       xex-link link-script <output.x> [--manifest PATH]

The first form converts a linked ELF into a XEX. The second writes the GNU ld
script that places the sections of a module for the manifest: code at the load
address plus one page (the first page holds the headers), each kind of section
on its own pages.

options:
  --manifest PATH         TOML file of XEX metadata; every key is optional
  --resource NAME=PATH    embed the file as the resource NAME (1 to 8 bytes,
                          case-sensitive), which XexGetModuleSection returns;
                          repeatable

manifest keys:
  title_id = INT          title id (default 0x00000000)
  media_id = INT          media id (default 0x00000000)
  load_address = INT      image base the ELF was linked at (default 0x82000000)
  version = \"VER\"         title version and export table version, dotted
                          a.b.c.d or \"0xHEX\" (default \"0.0.0.0\")
  base_version = \"VER\"    minimum loader version, same format (default \"0.0.0.0\")
  image_flags = [\"FLAG\"]  ImageFlags names or \"0xHEX\" (e.g. SMALL_PAGES; default [])
  module_flags = [\"FLAG\"] ModuleFlags names or \"0xHEX\" (default [\"TITLE\"])
  bounding_path = \"PATH\"  device path the loaded file must resolve to
  compress = BOOL         LZX-compress the image (Normal, 64 KB window; default false)
  encrypt = \"TARGET\"      encrypt payload with the \"devkit\" or \"retail\" master key
  tls_slots = INT         TLS slots in the TLS_INFO header; 0 omits it
                          (default 64 for titles, none for other modules)
  [imports]
  version = \"VER\"         imported module version (default \"2.0.21173.0\")
  min_version = \"VER\"     minimum imported module version (default \"2.0.1861.0\")
  [exports]               emit an export table
  module = \"NAME\"         module id to record: xboxkrnl.exe, xam.xex, xbdm.xex
  min_version = \"VER\"     oldest compatible version recorded in the export
                          table (default \"2.0.1861.0\")";

/// What to do.
pub enum Command {
    Link(Config),
    LinkScript { output: String, manifest: Manifest },
}

pub struct Config {
    pub input: String,
    pub output: String,
    pub manifest: Manifest,
    pub resources: Vec<ResourceFile>,
}

impl Command {
    /// Parse the process arguments. `--help` prints usage and exits.
    pub fn from_args() -> Result<Self> {
        let mut args = std::env::args().skip(1).peekable();
        let script = args.peek().is_some_and(|arg| arg == "link-script");
        if script {
            args.next();
        }

        let mut paths = Vec::new();
        let mut manifest_path = None;
        let mut resources = Vec::new();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--manifest" => {
                    let path = args
                        .next()
                        .ok_or_else(|| XexLinkError::MissingValue(arg.clone()).into_report())?;
                    manifest_path = Some(path);
                }
                "--resource" if !script => {
                    let value = args
                        .next()
                        .ok_or_else(|| XexLinkError::MissingValue(arg.clone()).into_report())?;
                    resources.push(ResourceFile::parse(&value)?);
                }
                "-h" | "--help" => {
                    println!("{USAGE}");
                    std::process::exit(0);
                }
                _ if arg.starts_with('-') => {
                    return Err(XexLinkError::UnexpectedArg(arg).into_report());
                }
                _ => paths.push(arg),
            }
        }

        let manifest = match manifest_path {
            Some(path) => {
                let text = io(&path, std::fs::read_to_string(&path))?;
                Manifest::parse(Path::new(&path), &text)
                    .map_err(|e| XexLinkError::Manifest(e).into_report())?
            }
            None => Manifest::default(),
        };

        let expected = if script { 1 } else { 2 };
        if let Some(extra) = paths.get(expected) {
            return Err(XexLinkError::UnexpectedArg(extra.clone()).into_report());
        }
        let mut paths = paths.into_iter();
        if script {
            let Some(output) = paths.next() else {
                return Err(XexLinkError::MissingPaths.into_report());
            };
            return Ok(Self::LinkScript { output, manifest });
        }

        let (Some(input), Some(output)) = (paths.next(), paths.next()) else {
            return Err(XexLinkError::MissingPaths.into_report());
        };
        Ok(Self::Link(Config {
            input,
            output,
            manifest,
            resources,
        }))
    }
}
