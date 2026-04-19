use std::borrow::Cow;

use heck::ToKebabCase;

use crate::{FileLayout, FilePage, PathBuf, PathSegment, Router, Segment, SegmentKind, Segments};

#[doc(hidden)]
pub struct FileRouter {
    inner: Router,
    file_root: &'static str,
    segments: Segments,
}

impl FileRouter {
    pub fn new(file_root: &'static str) -> Self {
        Self {
            file_root: Self::strip_module_suffix(file_root),
            inner: Router::new(),
            segments: Segments::new(),
        }
    }

    fn strip_module_suffix(file: &'static str) -> &'static str {
        let path = file.strip_suffix(".rs").unwrap_or(file);
        let path = path.strip_suffix("/mod").unwrap_or(path);
        path.strip_suffix("\\mod").unwrap_or(path)
    }

    fn canonical_module_path(&self, file: &'static str) -> &'static str {
        let path = file
            .strip_prefix(self.file_root)
            .expect("file must be under file router's file root");
        Self::strip_module_suffix(path)
    }

    pub fn segment(mut self, segment: Segment) -> Self {
        assert!(
            self.inner.is_empty(),
            "`segment` must be called before registering any resource"
        );
        self.segments
            .register(self.canonical_module_path(segment.file()), segment);
        self
    }

    fn file_to_path(&self, file: &'static str) -> PathBuf {
        let module_path = self.canonical_module_path(file);
        let mut path_buf = PathBuf::new();
        let mut current_index = 0;

        // Iterate over the folder structure. At each module level, check if there is a matching
        // [`Segment`] for that path specified by the user that overrides the default behavior.
        for component in module_path.split(&['/', '\\']).skip(1) {
            current_index += component.len() + 1;
            let segment = self.segments.get(&module_path[..current_index]);

            // A module is a group segment if it starts with "_" or a static segment otherwise,
            // unless this is overridden by the user.
            let kind = match segment.and_then(|segment| segment.kind()) {
                Some(kind) => *kind,
                None => match component.starts_with("_") {
                    true => SegmentKind::Group,
                    false => SegmentKind::Static,
                },
            };
            // Static segments are converted to kebab-case, other modules names are left as is.
            // This can also be overridden by the user.
            let name = match segment.and_then(|segment| segment.rename()) {
                Some(rename) => Cow::Borrowed(rename),
                None => match kind {
                    SegmentKind::Static => Cow::Owned(component.to_kebab_case()),
                    _ => Cow::Borrowed(component),
                },
            };

            let path_segment = match kind {
                SegmentKind::Static => PathSegment::Static(&name),
                SegmentKind::Group => PathSegment::Group(&name),
                SegmentKind::Param => PathSegment::Param(&name),
                SegmentKind::CatchAll => PathSegment::CatchAll(&name),
            };

            path_buf += path_segment;
        }
        path_buf
    }

    pub fn page(mut self, page: FilePage) -> Self {
        let file = page.file();
        let page = page.into_page(Cow::Owned(self.file_to_path(file)));
        self.inner = self.inner.page(page);
        self
    }

    pub fn layout(mut self, layout: FileLayout) -> Self {
        let file = layout.file();
        let layout = layout.into_layout(Cow::Owned(self.file_to_path(file)));
        self.inner = self.inner.layout(layout);
        self
    }

    pub fn discover(mut self) -> Self {
        for segment in inventory::iter::<Segment>().cloned() {
            self = self.segment(segment);
        }
        for page in inventory::iter::<FilePage>().cloned() {
            self = self.page(page);
        }
        for layout in inventory::iter::<FileLayout>().cloned() {
            self = self.layout(layout);
        }
        self.inner = self.inner.discover();
        self
    }
}

impl From<FileRouter> for Router {
    fn from(value: FileRouter) -> Self {
        value.inner
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use topcoat_view::runtime::View;

    use super::*;

    fn router(root: &'static str) -> FileRouter {
        FileRouter::new(root)
    }

    fn seg(file: &'static str, kind: Option<SegmentKind>, rename: Option<&'static str>) -> Segment {
        Segment::new(file, kind, rename.map(Cow::Borrowed))
    }

    // ── strip_module_suffix ──────────────────────────────────────────

    #[test]
    fn strip_rs_extension() {
        assert_eq!(
            FileRouter::strip_module_suffix("src/app/about.rs"),
            "src/app/about"
        );
    }

    #[test]
    fn strip_mod_rs() {
        assert_eq!(FileRouter::strip_module_suffix("src/app/mod.rs"), "src/app");
    }

    #[test]
    fn strip_mod_unix() {
        assert_eq!(FileRouter::strip_module_suffix("src/app/mod"), "src/app");
    }

    #[test]
    fn strip_mod_windows() {
        assert_eq!(FileRouter::strip_module_suffix("src\\app\\mod"), "src\\app");
    }

    #[test]
    fn strip_no_suffix() {
        assert_eq!(FileRouter::strip_module_suffix("src/app"), "src/app");
    }

    // ── file_to_path: basic static routes ────────────────────────────

    #[test]
    fn root_page() {
        let r = router("src/app/mod.rs");
        assert_eq!(r.file_to_path("src/app/mod.rs").to_string(), "");
    }

    #[test]
    fn simple_page() {
        let r = router("src/app/mod.rs");
        assert_eq!(r.file_to_path("src/app/about.rs").to_string(), "/about");
    }

    #[test]
    fn nested_page() {
        let r = router("src/app/mod.rs");
        assert_eq!(
            r.file_to_path("src/app/settings/profile.rs").to_string(),
            "/settings/profile"
        );
    }

    #[test]
    fn nested_mod() {
        let r = router("src/app/mod.rs");
        assert_eq!(
            r.file_to_path("src/app/settings/mod.rs").to_string(),
            "/settings"
        );
    }

    // ── file_to_path: kebab-case conversion ──────────────────────────

    #[test]
    fn static_segment_is_kebab_cased() {
        let r = router("src/app/mod.rs");
        assert_eq!(r.file_to_path("src/app/my_page.rs").to_string(), "/my-page");
    }

    #[test]
    fn nested_static_segments_are_kebab_cased() {
        let r = router("src/app/mod.rs");
        assert_eq!(
            r.file_to_path("src/app/user_settings/change_password.rs")
                .to_string(),
            "/user-settings/change-password"
        );
    }

    // ── file_to_path: group segments (underscore prefix) ─────────────

    #[test]
    fn group_segment() {
        let r = router("src/app/mod.rs");
        assert_eq!(
            r.file_to_path("src/app/_group/contact.rs").to_string(),
            "/(_group)/contact"
        );
    }

    #[test]
    fn group_mod() {
        let r = router("src/app/mod.rs");
        assert_eq!(
            r.file_to_path("src/app/_group/mod.rs").to_string(),
            "/(_group)"
        );
    }

    #[test]
    fn nested_groups() {
        let r = router("src/app/mod.rs");
        assert_eq!(
            r.file_to_path("src/app/_auth/_admin/dashboard.rs")
                .to_string(),
            "/(_auth)/(_admin)/dashboard"
        );
    }

    // ── file_to_path: segment overrides ──────────────────────────────

    #[test]
    fn override_static_to_param() {
        let r = router("src/app/mod.rs").segment(seg(
            "src/app/user_id.rs",
            Some(SegmentKind::Param),
            None,
        ));
        assert_eq!(
            r.file_to_path("src/app/user_id.rs").to_string(),
            "/{user_id}"
        );
    }

    #[test]
    fn override_static_to_catch_all() {
        let r = router("src/app/mod.rs").segment(seg(
            "src/app/rest.rs",
            Some(SegmentKind::CatchAll),
            None,
        ));
        assert_eq!(r.file_to_path("src/app/rest.rs").to_string(), "/{*rest}");
    }

    #[test]
    fn override_group_to_static() {
        let r = router("src/app/mod.rs").segment(seg(
            "src/app/_internal/mod.rs",
            Some(SegmentKind::Static),
            None,
        ));
        // Overridden to static, so kebab-case is applied to the "_internal" name.
        assert_eq!(
            r.file_to_path("src/app/_internal/page.rs").to_string(),
            "/internal/page"
        );
    }

    #[test]
    fn rename_segment() {
        let r = router("src/app/mod.rs").segment(seg("src/app/blog_post.rs", None, Some("posts")));
        assert_eq!(r.file_to_path("src/app/blog_post.rs").to_string(), "/posts");
    }

    #[test]
    fn rename_and_kind_override() {
        let r = router("src/app/mod.rs").segment(seg(
            "src/app/slug.rs",
            Some(SegmentKind::Param),
            Some("id"),
        ));
        assert_eq!(r.file_to_path("src/app/slug.rs").to_string(), "/{id}");
    }

    #[test]
    fn param_in_nested_path() {
        let r = router("src/app/mod.rs").segment(seg(
            "src/app/users/id/mod.rs",
            Some(SegmentKind::Param),
            None,
        ));
        assert_eq!(
            r.file_to_path("src/app/users/id/mod.rs").to_string(),
            "/users/{id}"
        );
        assert_eq!(
            r.file_to_path("src/app/users/id/settings.rs").to_string(),
            "/users/{id}/settings"
        );
    }

    #[test]
    fn catch_all_nested() {
        let r = router("src/app/mod.rs").segment(seg(
            "src/app/docs/path/mod.rs",
            Some(SegmentKind::CatchAll),
            None,
        ));
        assert_eq!(
            r.file_to_path("src/app/docs/path/mod.rs").to_string(),
            "/docs/{*path}"
        );
    }

    // ── file_to_path: multiple segment overrides ─────────────────────

    #[test]
    fn multiple_segments() {
        let r = router("src/app/mod.rs")
            .segment(seg(
                "src/app/users/id/mod.rs",
                Some(SegmentKind::Param),
                None,
            ))
            .segment(seg(
                "src/app/users/id/posts/post_id.rs",
                Some(SegmentKind::Param),
                None,
            ));
        assert_eq!(
            r.file_to_path("src/app/users/id/posts/post_id.rs")
                .to_string(),
            "/users/{id}/posts/{post_id}"
        );
    }

    // ── file_to_path: windows-style paths ────────────────────────────

    #[test]
    fn windows_separator() {
        let r = router("src\\app\\mod.rs");
        assert_eq!(r.file_to_path("src\\app\\about.rs").to_string(), "/about");
    }

    // ── segment ordering assertion ───────────────────────────────────

    #[test]
    #[should_panic(expected = "`segment` must be called before registering any resource")]
    fn segment_after_page_panics() {
        let r = router("src/app/mod.rs");
        // Register a page first, then try to add a segment.
        let page = FilePage::new("src/app/about.rs", || Box::pin(async { View::new("") }));
        r.page(page)
            .segment(seg("src/app/users.rs", Some(SegmentKind::Param), None));
    }
}
