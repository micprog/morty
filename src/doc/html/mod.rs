//! HTML documentation rendering

use super::*;
use anyhow::{Context as _, Result};
use std::{
    fs::write,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
mod static_files;

/// A HTML renderer.
pub struct Renderer<'a> {
    dir: &'a Path,
}

impl<'a> Renderer<'a> {
    /// Create a new HTML renderer.
    pub fn new(dir: &'a Path) -> Self {
        Self { dir }
    }

    pub fn render_header(&mut self, out: &mut impl Write) -> Result<()> {
        writeln!(out, "<html>")?;
        writeln!(
            out,
            "<link rel=\"stylesheet\" type=\"text/css\" href=\"static/rustdoc.css\">"
        )?;
        writeln!(
            out,
            "<link rel=\"stylesheet\" type=\"text/css\" href=\"static/light.css\">"
        )?;
        Ok(())
    }

    /// Render some documentation.
    pub fn render(&mut self, doc: &Doc) -> Result<()> {
        debug!("Render documentation");

        // Create the documentation directory.
        std::fs::create_dir_all(self.dir)
            .with_context(|| format!("Failed to create doc directory `{}`", self.dir.display()))?;

        let mut static_path = self.dir.join("static");
        std::fs::create_dir_all(&mut static_path)
            .with_context(|| format!("Failed to create doc directory `{}`", self.dir.display()))?;

        write(static_path.join("light.css"), static_files::LIGHT)?;
        write(static_path.join("rustdoc.css"), static_files::RUSTDOC_CSS)?;
        write(
            static_path.join("SourceSerifPro-Regular.ttf.woff"),
            static_files::source_serif_pro::REGULAR,
        )?;
        write(
            static_path.join("SourceSerifPro-Bold.ttf.woff"),
            static_files::source_serif_pro::BOLD,
        )?;
        write(
            static_path.join("SourceSerifPro-It.ttf.woff"),
            static_files::source_serif_pro::ITALIC,
        )?;

        write(
            static_path.join("SourceCodePro-Regular.woff"),
            static_files::source_code_pro::REGULAR,
        )?;
        write(
            static_path.join("SourceCodePro-Semibold.woff"),
            static_files::source_code_pro::SEMIBOLD,
        )?;

        write(
            static_path.join("FiraSans-Regular.woff"),
            static_files::fira_sans::REGULAR,
        )?;
        write(
            static_path.join("FiraSans-Medium.woff"),
            static_files::fira_sans::MEDIUM,
        )?;

        // Render the modules.
        for i in &doc.data.modules {
            self.render_module(i)
                .with_context(|| format!("Failed ro render module `{}`", i.name))?;
        }

        // Render the index.
        self.render_index(doc)
            .with_context(|| "Failed to render index")?;

        Ok(())
    }

    fn render_index(&mut self, doc: &Doc) -> Result<()> {
        let path = self.dir.join("index.html");
        debug!("Render index into `{}`", path.display());
        let mut out = File::create(path)?;

        self.render_header(&mut out)?;
        write!(out, "<body>\n")?;
        write!(out, "<section id=\"main\" class=\"content\">")?;
        write!(out, "<h1 class=\"fqn\">Documentation</h1>\n")?;

        self.render_contents(&doc.data, &mut out)?;

        write!(out, "</section>\n")?;
        write!(out, "</body>\n")?;
        write!(out, "</html>\n")?;

        Ok(())
    }

    fn render_module(&mut self, item: &ModuleItem) -> Result<()> {
        let path = self.path_to_module(&item.name);
        debug!("Render module `{}` into `{}`", item.name, path.display());
        let mut out = File::create(path)?;

        self.render_header(&mut out)?;
        write!(out, "<body>\n")?;
        write!(out, "<section id=\"main\" class=\"content\">")?;
        write!(
            out,
            "<h1 class=\"fqn\">Module <em>{}</em></h1>\n",
            item.name
        )?;

        write!(out, "<div class=\"docblock\">\n")?;
        self.render_doc(&item.doc, &mut out)?;
        write!(out, "</div>\n")?;

        self.render_contents(&item.content, &mut out)?;

        write!(out, "</section>\n")?;
        write!(out, "</body>\n")?;
        write!(out, "</html>\n")?;

        Ok(())
    }

    fn render_type(&mut self, item: &TypeItem) -> Result<()> {
        let path = self.path_to_type(&item.name);
        debug!("Render type `{}` into `{}`", item.name, path.display());
        let mut out = File::create(path)?;

        self.render_header(&mut out)?;
        write!(out, "<body>\n")?;
        write!(out, "<section id=\"main\" class=\"content\">")?;
        write!(
            out,
            "<h1 class=\"fqn\">Typedef <em>{}</em></h1>\n",
            item.name
        )?;

        write!(out, "<pre>typedef {} {};</pre>\n", item.ty, item.name)?;
        self.render_doc(&item.doc, &mut out)?;

        write!(out, "</section>\n")?;
        write!(out, "</body>\n")?;
        write!(out, "</html>\n")?;

        Ok(())
    }

    fn render_contents(&mut self, cx: &Context, out: &mut impl Write) -> Result<()> {
        if !cx.modules.is_empty() {
            write!(out, "<h2>Modules</h2>\n")?;
            write!(out, "<table>\n")?;
            for i in &cx.modules {
                write!(
                    out,
                    "<tr><td><a href=\"{}\">{}</a></td><td>",
                    self.subpath_to_module(&i.name),
                    i.name
                )?;
                self.render_headline_doc(&i.doc, out)?;
                write!(out, "</td></tr>")?;
            }
            write!(out, "</table>\n")?;
        }
        if !cx.params.is_empty() {
            write!(out, "<h2>Parameters</h2>\n")?;
            for i in &cx.params {
                write!(
                    out,
                    "<h3 id=\"impl\" class=\"impl\"><code class=\"in-band\">{}</code></h3>",
                    i.name
                )?;
                write!(out, "<div class=\"docblock\"\n>")?;
                self.render_doc(&i.doc, out)?;
                write!(out, "</div>")?;
            }
        }
        if !cx.ports.is_empty() {
            write!(out, "<h2>Ports</h2>\n")?;
            for i in &cx.ports {
                write!(
                    out,
                    "<h3 id=\"impl\" class=\"impl\"><code class=\"in-band\">{}</code></h3>",
                    i.name
                )?;
                write!(out, "<div class=\"docblock\"\n>")?;
                self.render_doc(&i.doc, out)?;
                write!(out, "</div>")?;
            }
        }
        if !cx.types.is_empty() {
            write!(out, "<h2>Types</h2>\n")?;
            write!(out, "<table>\n")?;
            for i in &cx.types {
                write!(
                    out,
                    "<tr><td><a href=\"{}\">{}</a></td><td>",
                    self.subpath_to_type(&i.name),
                    i.name
                )?;
                self.render_headline_doc(&i.doc, out)?;
                write!(out, "</td></tr>")?;
                self.render_type(i)?;
            }
            write!(out, "</table>\n")?;
        }
        if !cx.vars.is_empty() {
            write!(out, "<h2>Signals</h2>\n")?;
            for i in &cx.vars {
                write!(
                    out,
                    "<h3 id=\"impl\" class=\"impl\"><code class=\"in-band\">{}: {}</code></h3>",
                    i.name, i.ty
                )?;
                write!(out, "<div class=\"docblock\"\n>")?;
                self.render_doc(&i.doc, out)?;
                write!(out, "</div>")?;
            }
        }
        Ok(())
    }

    /// Render the headline markdown documentation.
    fn render_headline_doc(&mut self, doc: &str, out: &mut impl Write) -> Result<()> {
        let slice = doc.lines().next().unwrap_or("");
        let parser = pulldown_cmark::Parser::new_ext(slice, pulldown_cmark::Options::all());
        pulldown_cmark::html::write_html(out, parser)?;
        Ok(())
    }

    /// Render markdown documentation.
    fn render_doc(&mut self, doc: &str, out: &mut impl Write) -> Result<()> {
        let parser = pulldown_cmark::Parser::new_ext(doc, pulldown_cmark::Options::all());
        pulldown_cmark::html::write_html(out, parser)?;
        Ok(())
    }

    fn subpath_to_module(&self, name: &str) -> String {
        format!("module.{}.html", name)
    }

    fn subpath_to_type(&self, name: &str) -> String {
        format!("type.{}.html", name)
    }

    fn path_to_module(&self, name: &str) -> PathBuf {
        self.dir.join(self.subpath_to_module(name))
    }

    fn path_to_type(&self, name: &str) -> PathBuf {
        self.dir.join(self.subpath_to_type(name))
    }
}
