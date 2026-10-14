//! Archaeological Drawing Book Folder Resolution and Management (design 2.1 §2, §14(1)).
//!
//! Rule §14(1):
//! - The drawing book is defined as all drawings belonging to one excavation survey.
//! - It resides strictly inside `<Strata 조사 폴더>/도면`.
//! - The Strata survey file (`.gpkg`) remains strictly read-only (never modified or moved).

use std::path::{Path, PathBuf};

/// Status of the Strata survey GeoPackage (`.gpkg`) connection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StrataGpkgStatus {
    /// Connected to an existing Strata `.gpkg` file (read-only).
    Connected { filename: String, path: PathBuf },
    /// Expected a Strata `.gpkg` file but it was not found at the expected path.
    Missing { filename: String, expected_path: PathBuf },
    /// No Strata survey file linked.
    Disconnected,
}

#[allow(dead_code)]
impl StrataGpkgStatus {
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected { .. })
    }

    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing { .. })
    }
}

/// A resolved archaeological drawing book.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedDrawingBook {
    /// Drawing book / survey name (e.g. "광령리1").
    pub name: String,
    /// Root survey folder (e.g. `D:\발굴\광령리1`).
    pub survey_dir: PathBuf,
    /// Drawing book folder: strictly `<survey_dir>/도면` per §14(1).
    pub drawing_book_dir: PathBuf,
    /// Status of the Strata `.gpkg` link.
    pub strata_status: StrataGpkgStatus,
    /// Number of feature plan drawings (평면).
    pub plan_count: usize,
    /// Number of section / stratigraphy drawings (단면).
    pub section_count: usize,
    /// Number of artifact / finds drawings (유물).
    pub finds_count: usize,
    /// Number of report plate drawings (도판).
    pub plate_count: usize,
    /// Coordinate Reference System (e.g. "5186 중부원점").
    pub crs: String,
    /// Last opened timestamp or relative label (e.g. "오늘 07:04").
    pub last_opened: String,
}

#[allow(dead_code)]
impl ResolvedDrawingBook {
    /// Strictly resolves the drawing book location to `<Strata 조사 폴더>/도면` per §14(1).
    pub fn resolve(target: &Path) -> Self {
        let (survey_dir, explicit_gpkg) = if target.is_file() {
            let parent = target.parent().unwrap_or_else(|| Path::new("."));
            if target.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("gpkg")) {
                (parent.to_path_buf(), Some(target.to_path_buf()))
            } else if parent.file_name().is_some_and(|n| n == "도면") {
                (parent.parent().unwrap_or_else(|| Path::new(".")).to_path_buf(), None)
            } else {
                (parent.to_path_buf(), None)
            }
        } else if target.file_name().is_some_and(|n| n == "도면") {
            (target.parent().unwrap_or_else(|| Path::new(".")).to_path_buf(), None)
        } else {
            (target.to_path_buf(), None)
        };

        // Rule §14(1): Drawing book folder is strictly <survey_dir>/도면.
        let drawing_book_dir = survey_dir.join("도면");

        let name = survey_dir
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "도면집".to_string());

        let strata_status = if let Some(gpkg) = explicit_gpkg {
            let filename = gpkg.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            if gpkg.exists() {
                StrataGpkgStatus::Connected { filename, path: gpkg }
            } else {
                StrataGpkgStatus::Missing { filename, expected_path: gpkg }
            }
        } else {
            let mut detected = None;
            if let Ok(entries) = std::fs::read_dir(&survey_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("gpkg")) {
                        let fname = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                        detected = Some((fname, path));
                        break;
                    }
                }
            }
            match detected {
                Some((filename, path)) => StrataGpkgStatus::Connected { filename, path },
                None => StrataGpkgStatus::Disconnected,
            }
        };

        let (plan_count, section_count, finds_count, plate_count) = Self::scan_drawings(&drawing_book_dir);

        Self {
            name,
            survey_dir,
            drawing_book_dir,
            strata_status,
            plan_count,
            section_count,
            finds_count,
            plate_count,
            crs: "5186 중부원점".to_string(),
            last_opened: "오늘".to_string(),
        }
    }

    /// Scans the drawing book directory and categorizes drawing files into:
    /// (plan, section, finds, plate).
    pub fn scan_drawings(dir: &Path) -> (usize, usize, usize, usize) {
        let mut plan = 0;
        let mut section = 0;
        let mut finds = 0;
        let mut plate = 0;

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let is_drawing = p.is_file()
                    && p.extension().is_some_and(|ext| {
                        ext.eq_ignore_ascii_case("vectorcraft")
                            || ext.eq_ignore_ascii_case("drawcraft")
                            || ext.eq_ignore_ascii_case("trowel")
                    });
                if is_drawing {
                    let stem = p.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default();
                    if stem.contains("단면") || stem.contains("토층") {
                        section += 1;
                    } else if stem.contains("유물") || stem.contains("토기") {
                        finds += 1;
                    } else if stem.contains("도판") {
                        plate += 1;
                    } else {
                        plan += 1;
                    }
                }
            }
        }
        (plan, section, finds, plate)
    }
}

/// Recently accessed drawing books manager.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecentDrawingBooks {
    pub books: Vec<ResolvedDrawingBook>,
}

impl Default for RecentDrawingBooks {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl RecentDrawingBooks {
    pub fn new() -> Self {
        Self {
            books: Self::default_recents(),
        }
    }

    /// Default drawing books matching mockup §2:
    /// - 광령리1 (Connected to 광령리1.gpkg)
    /// - 중문동 2지점 (Disconnected)
    /// - 하가리 시굴 (Missing: 하가리.gpkg not found)
    pub fn default_recents() -> Vec<ResolvedDrawingBook> {
        vec![
            ResolvedDrawingBook {
                name: "광령리1".to_string(),
                survey_dir: PathBuf::from("D:\\발굴\\광령리1"),
                drawing_book_dir: PathBuf::from("D:\\발굴\\광령리1\\도면"),
                strata_status: StrataGpkgStatus::Connected {
                    filename: "광령리1.gpkg".to_string(),
                    path: PathBuf::from("D:\\발굴\\광령리1\\광령리1.gpkg"),
                },
                plan_count: 4,
                section_count: 3,
                finds_count: 8,
                plate_count: 2,
                crs: "5186 중부원점".to_string(),
                last_opened: "오늘 07:04".to_string(),
            },
            ResolvedDrawingBook {
                name: "중문동 2지점".to_string(),
                survey_dir: PathBuf::from("D:\\발굴\\중문동"),
                drawing_book_dir: PathBuf::from("D:\\발굴\\중문동\\도면"),
                strata_status: StrataGpkgStatus::Disconnected,
                plan_count: 2,
                section_count: 5,
                finds_count: 0,
                plate_count: 0,
                crs: "5186 중부원점".to_string(),
                last_opened: "어제".to_string(),
            },
            ResolvedDrawingBook {
                name: "하가리 시굴".to_string(),
                survey_dir: PathBuf::from("D:\\발굴\\하가리"),
                drawing_book_dir: PathBuf::from("D:\\발굴\\하가리\\도면"),
                strata_status: StrataGpkgStatus::Missing {
                    filename: "하가리.gpkg".to_string(),
                    expected_path: PathBuf::from("D:\\발굴\\하가리\\하가리.gpkg"),
                },
                plan_count: 1,
                section_count: 1,
                finds_count: 0,
                plate_count: 1,
                crs: "5186 중부원점".to_string(),
                last_opened: "2026-09-28".to_string(),
            },
        ]
    }

    /// Removes a drawing book at the given index.
    pub fn remove_at(&mut self, index: usize) {
        if index < self.books.len() {
            self.books.remove(index);
        }
    }

    /// Removes a drawing book by directory path.
    pub fn remove_by_path(&mut self, path: &Path) {
        self.books.retain(|b| b.drawing_book_dir != path && b.survey_dir != path);
    }

    /// Adds a book to the top of the recents list, deduplicating existing entries.
    pub fn add(&mut self, book: ResolvedDrawingBook) {
        self.books.retain(|b| b.drawing_book_dir != book.drawing_book_dir);
        self.books.insert(0, book);
    }
}
