use crate::flags::{DirOrderFlag, Flags, SortFlag, SortOrder};
use crate::meta::{FileType, Meta};
use std::cmp::Ordering;

pub type Sorter = Box<dyn Fn(&Meta, &Meta) -> Ordering>;

pub fn create_sorter(flags: &Flags) -> Sorter {
    let mut sorters: Vec<(SortOrder, Sorter)> = vec![];
    match flags.directory_order {
        DirOrderFlag::First => {
            sorters.push((SortOrder::Default, Box::new(with_dirs_first)));
        }
        DirOrderFlag::Last => {
            sorters.push((SortOrder::Reverse, Box::new(with_dirs_first)));
        }
        DirOrderFlag::None => {}
    };
    let other_sort = match flags.sort_by {
        SortFlag::Name => by_name,
        SortFlag::Size => by_size,
        SortFlag::Time => by_date,
    };
    sorters.push((flags.sort_order, Box::new(other_sort)));

    Box::new(move |a, b| {
        for (direction, sorter) in sorters.iter() {
            match (sorter)(a, b) {
                Ordering::Equal => continue,
                ordering => {
                    return match direction {
                        SortOrder::Reverse => ordering.reverse(),
                        SortOrder::Default => ordering,
                    }
                }
            }
        }
        Ordering::Equal
    })
}

fn with_dirs_first(a: &Meta, b: &Meta) -> Ordering {
    match (a.file_type, b.file_type) {
        (FileType::Directory { .. }, FileType::Directory { .. }) => Ordering::Equal,
        (FileType::Directory { .. }, FileType::SymLink { is_dir: true }) => Ordering::Equal,
        (FileType::SymLink { is_dir: true }, FileType::Directory { .. }) => Ordering::Equal,
        (FileType::Directory { .. }, _) => Ordering::Less,
        (_, FileType::Directory { .. }) => Ordering::Greater,
        (FileType::SymLink { is_dir: true }, _) => Ordering::Less,
        (_, FileType::SymLink { is_dir: true }) => Ordering::Greater,
        _ => Ordering::Equal,
    }
}

fn by_size(a: &Meta, b: &Meta) -> Ordering {
    b.size.get_bytes().cmp(&a.size.get_bytes())
}

fn by_name(a: &Meta, b: &Meta) -> Ordering {
    a.name.cmp(&b.name)
}

fn by_date(a: &Meta, b: &Meta) -> Ordering {
    b.date.cmp(&a.date).then(a.name.cmp(&b.name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flags::Flags;
    use std::fs::{create_dir, File};
    use std::process::Command;
    use tempfile::tempdir;

    #[test]
    fn test_sort_create_sorter_by_name_with_dirs_first() {
        let tmp_dir = tempdir().expect("failed to create temp dir");

        // Create the file;
        let path_a = tmp_dir.path().join("zzz");
        File::create(&path_a).expect("failed to create file");
        let meta_a = Meta::from_path(&path_a, false).expect("failed to get meta");

        // Create a dir;
        let path_z = tmp_dir.path().join("aaa");
        create_dir(&path_z).expect("failed to create dir");
        let meta_z = Meta::from_path(&path_z, false).expect("failed to get meta");

        let mut flags = Flags::default();
        flags.directory_order = DirOrderFlag::First;

        //  Sort with the dirs first
        let sorter = create_sorter(&flags);
        assert_eq!((sorter)(&meta_a, &meta_z), Ordering::Greater);

        //  Sort with the dirs first (the dirs stay first)
        flags.sort_order = SortOrder::Reverse;

        let sorter = create_sorter(&flags);
        assert_eq!((sorter)(&meta_a, &meta_z), Ordering::Greater);
    }

    #[test]
    fn test_sort_create_sorter_by_name_with_files_first() {
        let tmp_dir = tempdir().expect("failed to create temp dir");

        // Create the file;
        let path_a = tmp_dir.path().join("zzz");
        File::create(&path_a).expect("failed to create file");
        let meta_a = Meta::from_path(&path_a, false).expect("failed to get meta");

        // Create a dir;
        let path_z = tmp_dir.path().join("aaa");
        create_dir(&path_z).expect("failed to create dir");
        let meta_z = Meta::from_path(&path_z, false).expect("failed to get meta");

        let mut flags = Flags::default();
        flags.directory_order = DirOrderFlag::Last;

        // Sort with file first
        let sorter = create_sorter(&flags);
        assert_eq!((sorter)(&meta_a, &meta_z), Ordering::Less);

        // Sort with file first reversed (thie files stay first)
        let sorter = create_sorter(&flags);
        assert_eq!((sorter)(&meta_a, &meta_z), Ordering::Less);
    }

    #[test]
    fn test_sort_create_sorter_by_name_unordered() {
        let tmp_dir = tempdir().expect("failed to create temp dir");

        // Create the file;
        let path_a = tmp_dir.path().join("aaa");
        File::create(&path_a).expect("failed to create file");
        let meta_a = Meta::from_path(&path_a, false).expect("failed to get meta");

        // Create a dir;
        let path_z = tmp_dir.path().join("zzz");
        create_dir(&path_z).expect("failed to create dir");
        let meta_z = Meta::from_path(&path_z, false).expect("failed to get meta");

        let mut flags = Flags::default();
        flags.directory_order = DirOrderFlag::None;

        // Sort by name unordered
        let sorter = create_sorter(&flags);
        assert_eq!((sorter)(&meta_a, &meta_z), Ordering::Less);

        // Sort by name unordered
        flags.sort_order = SortOrder::Reverse;

        let sorter = create_sorter(&flags);
        assert_eq!((sorter)(&meta_a, &meta_z), Ordering::Greater);
    }

    #[test]
    fn test_sort_create_sorter_by_name_unordered_2() {
        let tmp_dir = tempdir().expect("failed to create temp dir");

        // Create the file;
        let path_a = tmp_dir.path().join("zzz");
        File::create(&path_a).expect("failed to create file");
        let meta_a = Meta::from_path(&path_a, false).expect("failed to get meta");

        // Create a dir;
        let path_z = tmp_dir.path().join("aaa");
        create_dir(&path_z).expect("failed to create dir");
        let meta_z = Meta::from_path(&path_z, false).expect("failed to get meta");

        let mut flags = Flags::default();
        flags.directory_order = DirOrderFlag::None;

        // Sort by name unordered
        let sorter = create_sorter(&flags);
        assert_eq!((sorter)(&meta_a, &meta_z), Ordering::Greater);

        // Sort by name unordered reversed
        flags.sort_order = SortOrder::Reverse;

        let sorter = create_sorter(&flags);
        assert_eq!((sorter)(&meta_a, &meta_z), Ordering::Less);
    }

    #[test]
    fn test_sort_create_sorter_by_time() {
        let tmp_dir = tempdir().expect("failed to create temp dir");

        // Create the file;
        let path_a = tmp_dir.path().join("aaa");
        File::create(&path_a).expect("failed to create file");
        let meta_a = Meta::from_path(&path_a, false).expect("failed to get meta");

        // Create the file;
        let path_z = tmp_dir.path().join("zzz");
        File::create(&path_z).expect("failed to create file");

        #[cfg(unix)]
        let success = Command::new("touch")
            .arg("-t")
            .arg("198511160000")
            .arg(&path_z)
            .status()
            .unwrap()
            .success();

        #[cfg(windows)]
        let success = Command::new("powershell")
            .arg("-Command")
            .arg("$(Get-Item")
            .arg(&path_z)
            .arg(").lastwritetime=$(Get-Date \"11/16/1985\")")
            .status()
            .unwrap()
            .success();

        assert_eq!(true, success, "failed to change file timestamp");
        let meta_z = Meta::from_path(&path_z, false).expect("failed to get meta");

        let mut flags = Flags::default();
        flags.sort_by = SortFlag::Time;

        // Sort by time
        let sorter = create_sorter(&flags);
        assert_eq!((sorter)(&meta_a, &meta_z), Ordering::Less);

        // Sort by time reversed
        flags.sort_order = SortOrder::Reverse;
        let sorter = create_sorter(&flags);
        assert_eq!((sorter)(&meta_a, &meta_z), Ordering::Greater);
    }
}
