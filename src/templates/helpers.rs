//! Handlebars helpers.

#![allow(missing_docs)]

use drone_config::format_size;
use handlebars::{
    handlebars_helper, Context, Handlebars, Helper, HelperDef, HelperResult, JsonValue, Output,
    PathAndJson, RenderContext, RenderError, Renderable, ScopedJson,
};
use std::{collections::HashMap, sync::Mutex};

thread_local! {
    static VARS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

handlebars_helper!(addr: |x: u64| format!("0x{:08x}", x));

handlebars_helper!(size: |x: u64| format_size(x as u32));

pub struct Set;

impl HelperDef for Set {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
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
        _r: &'reg Handlebars,
        _ctx: &'rc Context,
        _rc: &mut RenderContext<'reg>,
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

pub struct IfIncludes;

impl HelperDef for IfIncludes {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let value = h
            .param(0)
            .ok_or_else(|| RenderError::new("missing parameter"))?
            .render();
        let result = h
            .params()
            .iter()
            .skip(1)
            .map(PathAndJson::render)
            .any(|param| param == value);
        match if result { h.template() } else { h.inverse() } {
            Some(t) => t.render(r, ctx, rc, out),
            None => Ok(()),
        }
    }
}

/// Register all helpers.
pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("set", Box::new(Set));
    handlebars.register_helper("get", Box::new(Get));
    handlebars.register_helper("addr", Box::new(addr));
    handlebars.register_helper("size", Box::new(size));
    handlebars.register_helper("if-includes", Box::new(IfIncludes));
}

/// Clears all variables.
pub fn clear_vars() {
    VARS.with(|vars| vars.lock().unwrap().clear())
}
