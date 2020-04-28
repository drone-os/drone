//! Handlebars helpers.

#![allow(missing_docs)]

use drone_config::format_size;
use handlebars::{
    handlebars_helper, Context, Handlebars, Helper, HelperDef, HelperResult, JsonValue, Output,
    PathAndJson, RenderContext, RenderError, Renderable, ScopedJson,
};
use regex::Regex;
use std::{collections::HashMap, sync::Mutex};

thread_local! {
    static VARS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

handlebars_helper!(addr: |num: u64| format!("0x{:08x}", num));

handlebars_helper!(size: |num: u64| format_size(num as u32));

pub struct Set;

impl HelperDef for Set {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars<'_>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        _out: &mut dyn Output,
    ) -> HelperResult {
        let name = h
            .param(0)
            .ok_or_else(|| RenderError::new("missing parameter"))?
            .value()
            .as_str()
            .ok_or_else(|| RenderError::new("parameter must be a string"))?;
        let value = h
            .template()
            .ok_or_else(|| RenderError::new("missing inner template"))?
            .renders(r, ctx, rc)?;
        VARS.with(|vars| vars.lock().unwrap().insert(name.to_owned(), value));
        Ok(())
    }
}

pub struct Get;

impl HelperDef for Get {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _r: &'reg Handlebars<'_>,
        _ctx: &'rc Context,
        _rc: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let name = h
            .param(0)
            .ok_or_else(|| RenderError::new("missing parameter"))?
            .value()
            .as_str()
            .ok_or_else(|| RenderError::new("parameter must be a string"))?;
        match VARS.with(|vars| vars.lock().unwrap().get(name).cloned()) {
            Some(value) => Ok(Some(ScopedJson::Derived(JsonValue::from(value)))),
            None => Ok(None),
        }
    }
}

pub struct Replace;

impl HelperDef for Replace {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _r: &'reg Handlebars<'_>,
        _ctx: &'rc Context,
        _rc: &mut RenderContext<'reg, 'rc>,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let text = h
            .param(0)
            .ok_or_else(|| RenderError::new(format!("missing parameter {}", 0)))?
            .value()
            .as_str()
            .ok_or_else(|| RenderError::new(format!("parameter {} must be a string", 0)))?;
        let regex = h
            .param(1)
            .ok_or_else(|| RenderError::new(format!("missing parameter {}", 1)))?
            .value()
            .as_str()
            .ok_or_else(|| RenderError::new(format!("parameter {} must be a string", 1)))?;
        let replace = h
            .param(2)
            .ok_or_else(|| RenderError::new(format!("missing parameter {}", 2)))?
            .value()
            .as_str()
            .ok_or_else(|| RenderError::new(format!("parameter {} must be a string", 2)))?;
        let regex = Regex::new(regex).expect("invalid regex");
        let text = regex.replace(text, replace);
        Ok(Some(ScopedJson::Derived(JsonValue::from(text))))
    }
}

pub struct IfIncludes;

impl HelperDef for IfIncludes {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars<'_>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let value = h.param(0).ok_or_else(|| RenderError::new("missing parameter"))?.render();
        let result = h.params().iter().skip(1).map(PathAndJson::render).any(|param| param == value);
        match if result { h.template() } else { h.inverse() } {
            Some(t) => t.render(r, ctx, rc, out),
            None => Ok(()),
        }
    }
}

/// Register all helpers.
pub fn register(handlebars: &mut Handlebars<'_>) {
    handlebars.register_helper("set", Box::new(Set));
    handlebars.register_helper("get", Box::new(Get));
    handlebars.register_helper("addr", Box::new(addr));
    handlebars.register_helper("size", Box::new(size));
    handlebars.register_helper("replace", Box::new(Replace));
    handlebars.register_helper("if-includes", Box::new(IfIncludes));
}

/// Clears all variables.
pub fn clear_vars() {
    VARS.with(|vars| vars.lock().unwrap().clear())
}
