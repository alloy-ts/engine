use napi_derive::napi;

/// Predefined QuickJS atoms.
#[napi]
pub enum PredefinedAtom {
    Null = 0,
    False = 1,
    True = 2,
    Length = 3,
    Prototype = 4,
    Constructor = 5,
}

/// NAPI wrapper around QuickJS Atom operations.
#[napi(js_name = "Atom")]
pub struct JsAtom {
    name: String,
}

#[napi]
impl JsAtom {
    #[napi(constructor)]
    pub fn new(name: String) -> Self {
        JsAtom { name }
    }

    #[napi(js_name = "from_str")]
    pub fn from_str(name: String) -> Self {
        JsAtom { name }
    }

    #[napi(js_name = "of_type")]
    pub fn of_type(predefined: PredefinedAtom) -> Self {
        let name = match predefined {
            PredefinedAtom::Null => "null",
            PredefinedAtom::False => "false",
            PredefinedAtom::True => "true",
            PredefinedAtom::Length => "length",
            PredefinedAtom::Prototype => "prototype",
            PredefinedAtom::Constructor => "constructor",
        };
        JsAtom {
            name: name.to_string(),
        }
    }

    #[napi(js_name = "to_string")]
    pub fn to_string(&self) -> String {
        self.name.clone()
    }

    #[napi(js_name = "toString")]
    pub fn to_string_camel(&self) -> String {
        self.to_string()
    }
}
