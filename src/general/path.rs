use std::fmt::Debug;

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedNameSegment {
    pub ident: String,
    pub template_spec: Vec<QualifiedName>,
}

impl std::fmt::Display for QualifiedNameSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl QualifiedNameSegment {
    pub fn is_template(&self) -> bool {
        return !self.template_spec.is_empty();
    }

    pub fn mangle(&self) -> String {
        let mut template_extra = String::new();
        for a in &self.template_spec {
            template_extra.push_str("_");
            template_extra = a.mangle();
        }
        return format!("_mange_{}_", self.ident);
    }
}

impl Debug for QualifiedNameSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.template_spec.len() > 0 {
            let mut template = String::new();
            for spec in &self.template_spec {
                template += &format!("{spec:?}, ");
            }
            write!(f, "{}<{template}>", self.ident)
        } else {
            write!(f, "{}", self.ident)
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedName {
    pub v: Vec<QualifiedNameSegment>,
}

impl std::fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Debug for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.v.iter();
        if let Some(s) = iter.next() {
            write!(f, "{s:?}")?;
        }
        for segment in iter {
            write!(f, "::{segment:?}")?;
        }
        return Ok(());
    }
}

impl<I> From<I> for QualifiedNameSegment
where
    I: Into<String>,
{
    fn from(value: I) -> Self {
        let ident = value.into();
        return QualifiedNameSegment {
            ident,
            template_spec: vec![],
        };
    }
}

impl QualifiedName {
    pub fn add_segment<I: Into<QualifiedNameSegment>>(&mut self, s: I) {
        let ident = s.into();
        self.v.push(ident);
    }

    pub fn get(&self, index: usize) -> &QualifiedNameSegment {
        return &self.v[index];
    }

    pub fn new() -> Self {
        Self { v: vec![] }
    }

    pub fn pop_front(&mut self) {
        self.v.remove(0);
    }

    pub fn len(&self) -> usize {
        return self.v.len();
    }

    pub fn mangle(&self) -> String {
        let mut iter = self.v.iter();
        let mut s = iter.next().unwrap().mangle();
        for a in iter {
            s.push_str(&a.mangle());
        }

        return s;
    }
}

// isn't impl<T> AsRef<T> for T { ... } a default implementation?
impl AsRef<QualifiedName> for QualifiedName {
    fn as_ref(&self) -> &QualifiedName {
        return self;
    }
}

impl<T> From<T> for QualifiedName
where
    T: Into<QualifiedNameSegment>,
{
    fn from(value: T) -> Self {
        let s = value.into();
        return Self { v: vec![s] };
    }
}
