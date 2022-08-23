//! HTML documentation rendering

use super::*;
use anyhow::{Context as _, Result};
use chrono::Local;
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
        writeln!(
            out,
            "<!-- Compiled by morty-{} / {} -->\n",
            env!("CARGO_PKG_VERSION"),
            Local::now()
        )
        .unwrap();
        writeln!(out, "<html>")?;
        writeln!(
            out,
            "<link rel=\"stylesheet\" type=\"text/css\" href=\"static/rustdoc.css\">"
        )?;
        writeln!(
            out,
            "<link rel=\"stylesheet\" type=\"text/css\" href=\"static/light.css\">"
        )?;
        writeln!(
            out,
            "<link rel=\"stylesheet\" type=\"text/css\" href=\"static/svdoc.css\">"
        )?;
        Ok(())
    }

    /// Render some documentation.
    pub fn render(&mut self, doc: &Doc) -> Result<()> {
        debug!("Render documentation");

        // Create the documentation directory.
        std::fs::create_dir_all(self.dir)
            .with_context(|| format!("Failed to create doc directory `{}`", self.dir.display()))?;

        // Write the static files.
        let mut static_path = self.dir.join("static");
        std::fs::create_dir_all(&mut static_path)
            .with_context(|| format!("Failed to create doc directory `{}`", self.dir.display()))?;

        write(static_path.join("light.css"), static_files::LIGHT)?;
        write(static_path.join("rustdoc.css"), static_files::RUSTDOC_CSS)?;
        write(static_path.join("svdoc.css"), static_files::SVDOC_CSS)?;
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
        writeln!(out, "<body>")?;
        write!(out, "<section id=\"main\" class=\"content\">")?;
        writeln!(out, "<h1 class=\"fqn\">Documentation</h1>")?;

        self.render_contents(&doc.data, &mut out)?;

        writeln!(out, "</section>")?;
        writeln!(out, "</body>")?;
        writeln!(out, "</html>")?;

        Ok(())
    }

    fn render_package(&mut self, item: &PackageItem) -> Result<()> {
        let path = self.path_to_package(&item.name);
        debug!("Render package `{}` into `{}`", item.name, path.display());
        let mut out = File::create(path)?;

        self.render_header(&mut out)?;
        writeln!(out, "<body>")?;
        write!(out, "<section id=\"main\" class=\"content\">")?;
        writeln!(
            out,
            "<h1 class=\"fqn\">Package <a class=\"package\">{}</a></h1>",
            item.name
        )?;

        writeln!(out, "<div class=\"docblock\">")?;
        self.render_doc(&item.doc, &mut out)?;
        writeln!(out, "</div>")?;

        self.render_contents(&item.content, &mut out)?;

        writeln!(out, "</section>")?;
        writeln!(out, "</body>")?;
        writeln!(out, "</html>")?;

        Ok(())
    }

    fn render_module(&mut self, item: &ModuleItem) -> Result<()> {
        let path = self.path_to_module(&item.name);
        debug!("Render module `{}` into `{}`", item.name, path.display());
        let mut out = File::create(path)?;

        self.render_header(&mut out)?;
        writeln!(out, "<body>")?;
        write!(out, "<section id=\"main\" class=\"content\">")?;
        writeln!(
            out,
            "<h1 class=\"fqn\">Module <a class=\"module\">{}</a></h1>",
            item.name
        )?;

        writeln!(out, "<div class=\"docblock\">")?;
        self.render_doc(&item.doc, &mut out)?;
        writeln!(out, "</div>")?;

        self.render_contents(&item.content, &mut out)?;

        writeln!(out, "</section>")?;
        writeln!(out, "</body>")?;
        writeln!(out, "</html>")?;

        Ok(())
    }

    fn render_type(&mut self, item: &TypeItem) -> Result<()> {
        let path = self.path_to_type(&item.name);
        debug!("Render type `{}` into `{}`", item.name, path.display());
        let mut out = File::create(path)?;

        self.render_header(&mut out)?;
        writeln!(out, "<body>")?;
        write!(out, "<section id=\"main\" class=\"content\">")?;
        writeln!(
            out,
            "<h1 class=\"fqn\">Typedef <a class=\"type\">{}</a></h1>",
            item.name
        )?;

        writeln!(out, "<pre>typedef {} {};</pre>", item.ty, item.name)?;
        self.render_doc(&item.doc, &mut out)?;

        writeln!(out, "</section>")?;
        writeln!(out, "</body>")?;
        writeln!(out, "</html>")?;

        Ok(())
    }

    fn render_contents(&mut self, cx: &Context, out: &mut impl Write) -> Result<()> {
        if !cx.packages.is_empty() {
            writeln!(out, "<h2 id=\"packages\">Packages</h2>")?;
            writeln!(out, "<table>")?;
            for i in &cx.packages {
                write!(
                    out,
                    "<tr><td><a class=\"package\" href=\"{}\">{}</a></td><td>",
                    self.subpath_to_package(&i.name),
                    i.name
                )?;
                self.render_headline_doc(&i.doc, out)?;
                write!(out, "</td></tr>")?;
                self.render_package(i)
                    .with_context(|| format!("Failed ro render package `{}`", i.name))?;
            }
            writeln!(out, "</table>")?;
        }
        if !cx.modules.is_empty() {
            writeln!(out, "<h2 id=\"modules\">Modules</h2>")?;
            writeln!(out, "<table>")?;
            for i in &cx.modules {
                write!(
                    out,
                    "<tr><td><a class=\"module\" href=\"{}\">{}</a></td><td>",
                    self.subpath_to_module(&i.name),
                    i.name
                )?;
                self.render_headline_doc(&i.doc, out)?;
                write!(out, "</td></tr>")?;
                self.render_module(i)
                    .with_context(|| format!("Failed ro render module `{}`", i.name))?;
            }
            writeln!(out, "</table>")?;
        }
        if !cx.params.is_empty() {
            writeln!(out, "<h2 id=\"parameters\" class=\"section-header\"><a href=\"#parameters\">Parameters</a></h2>")?;
            for i in &cx.params {
                write!(
                    out,
                    "<h3 id=\"{2}\" class=\"impl\"><code class=\"in-band\"><a href=\"#{2}\">{0}</a><span class=\"type-annotation\">: {1}</span></code></h3>",
                    i.name,
                    i.ty,
                    i.html_id(),
                )?;
                writeln!(out, "<div class=\"docblock\">")?;
                self.render_doc(&i.doc, out)?;
                write!(out, "</div>")?;
            }
        }
        if !cx.ports.is_empty() {
            writeln!(
                out,
                "<h2 id=\"ports\" class=\"section-header\"><a href=\"#ports\">Ports</a></h2>"
            )?;
            for i in &cx.ports {
                write!(
                    out,
                    "<h3 id=\"{2}\" class=\"impl\"><code class=\"in-band\"><a href=\"#{2}\">{0}</a><span class=\"type-annotation\">: {1}</span></code></h3>",
                    i.name,
                    i.ty,
                    i.html_id(),
                )?;
                writeln!(out, "<div class=\"docblock\">")?;
                self.render_doc(&i.doc, out)?;
                write!(out, "</div>")?;
            }
        }
        if !cx.types.is_empty() {
            writeln!(
                out,
                "<h2 id=\"types\" class=\"section-header\"><a href=\"#types\">Types<a></h2>"
            )?;
            writeln!(out, "<table>")?;
            for i in &cx.types {
                write!(
                    out,
                    "<tr><td><a class=\"type\" href=\"{}\">{}</a></td><td>",
                    self.subpath_to_type(&i.name),
                    i.name
                )?;
                self.render_headline_doc(&i.doc, out)?;
                write!(out, "</td></tr>")?;
                self.render_type(i)
                    .with_context(|| format!("Failed ro render type `{}`", i.name))?;
            }
            writeln!(out, "</table>")?;
        }
        if !cx.vars.is_empty() {
            writeln!(
                out,
                "<h2 id=\"signals\" class=\"section-header\"><a href=\"#signals\">Signals</a></h2>"
            )?;
            for i in &cx.vars {
                write!(
                    out,
                    "<h3 id=\"{2}\" class=\"impl\"><code class=\"in-band\"><a href=\"#{2}\">{0}</a><span class=\"type-annotation\">: {1}</span></code></h3>",
                    i.name,
                    i.ty,
                    i.html_id(),
                )?;
                writeln!(out, "<div class=\"docblock\">")?;
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

    fn subpath_to_package(&self, name: &str) -> String {
        format!("package.{}.html", name)
    }

    fn subpath_to_module(&self, name: &str) -> String {
        format!("module.{}.html", name)
    }

    fn subpath_to_type(&self, name: &str) -> String {
        format!("type.{}.html", name)
    }

    fn path_to_package(&self, name: &str) -> PathBuf {
        self.dir.join(self.subpath_to_package(name))
    }

    fn path_to_module(&self, name: &str) -> PathBuf {
        self.dir.join(self.subpath_to_module(name))
    }

    fn path_to_type(&self, name: &str) -> PathBuf {
        self.dir.join(self.subpath_to_type(name))
    }
}

/// HTML identifier (value of `id` field).
trait Id {
    fn html_id(&self) -> String;
}

impl Id for ParamItem {
    fn html_id(&self) -> String {
        format!("parameter.{}", self.name)
    }
}

impl Id for PortItem {
    fn html_id(&self) -> String {
        format!("port.{}", self.name)
    }
}

impl Id for TypeItem {
    fn html_id(&self) -> String {
        format!("type.{}", self.name)
    }
}

impl Id for VarItem {
    fn html_id(&self) -> String {
        format!("signal.{}", self.name)
    }
}
