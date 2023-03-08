use gimli::{AttributeValue, DebugAbbrev, DebugInfo, DebugStr, EndianSlice, RunTimeEndian};
use object::{File, Object, ObjectSection};
use std::borrow::{Borrow, Cow};
use std::collections::BTreeSet;
use typed_arena::Arena;

pub(crate) fn parse_go_packages(obj: &File) -> Vec<String> {
    let endian = if obj.is_little_endian() {
        RunTimeEndian::Little
    } else {
        RunTimeEndian::Big
    };

    let arena = Arena::new();

    fn load_section<'a, 'file, 'input, S, Endian>(
        arena: &'a Arena<Cow<'file, [u8]>>,
        file: &'file File<'input>,
        endian: Endian,
    ) -> S
    where
        S: gimli::Section<EndianSlice<'a, Endian>>,
        Endian: gimli::Endianity + Send + Sync,
        'file: 'input,
        'a: 'file,
    {
        let data = match file.section_by_name(S::section_name()) {
            Some(ref section) => section
                .uncompressed_data()
                .unwrap_or(Cow::Borrowed(&[][..])),
            None => Cow::Borrowed(&[][..]),
        };
        let data_ref = (*arena.alloc(data)).borrow();
        S::from(gimli::EndianSlice::new(data_ref, endian))
    }

    let debug_abbrev = &load_section(&arena, obj, endian);
    let debug_info = &load_section(&arena, obj, endian);
    let debug_str = &load_section(&arena, obj, endian);

    return collect_go_packages(debug_info, debug_abbrev, debug_str);
}

fn collect_go_packages(
    debug_info: &DebugInfo<EndianSlice<RunTimeEndian>>,
    debug_abbrev: &DebugAbbrev<EndianSlice<RunTimeEndian>>,
    debug_str: &DebugStr<EndianSlice<RunTimeEndian>>,
) -> Vec<String> {
    let mut go_packages = BTreeSet::new();
    let mut units_iter = debug_info.units();

    loop {
        match units_iter.next() {
            Ok(Some(header)) => {
                let abbrevs = header.abbreviations(debug_abbrev).unwrap();
                let mut tree = header.entries_tree(&abbrevs, None).unwrap();
                let root = tree.root().unwrap();
                let entry = root.entry();
                let language = entry.attr_value(gimli::DW_AT_language).unwrap();
                if language.is_none() {
                    continue;
                }
                let language = language.unwrap();
                if language != AttributeValue::Language(gimli::DW_LANG_Go) {
                    continue;
                }
                let name = entry.attr_value(gimli::DW_AT_name).unwrap();
                if name.is_none() {
                    continue;
                }
                let name = name.unwrap();
                let parsed_name = name.string_value(&debug_str);
                if parsed_name.is_none() {
                    continue;
                }
                let parsed_name = parsed_name.unwrap();
                let utf8_name = parsed_name.to_string();
                if utf8_name.is_err() {
                    continue;
                }
                let utf8_name = utf8_name.unwrap();
                go_packages.insert(utf8_name.to_string());
            }
            Ok(None) => break,
            Err(e) => {
                eprintln!("Error parsing debug info: {}", e);
                break;
            }
        }
    }

    return go_packages.into_iter().collect();
}
