use syn;

// An iterator yielding all conrod attributes in the given attributes.
pub struct ConrodAttrs<I> {
    attrs: I,
}

pub fn conrod_attrs<'a, I>(attrs: I) -> ConrodAttrs<I::IntoIter>
    where I: IntoIterator<Item=&'a syn::Attribute>,
{
    ConrodAttrs { attrs: attrs.into_iter() }
}

impl<'a, I> Iterator for ConrodAttrs<I>
    where I: Iterator<Item=&'a syn::Attribute>,
{
    type Item = &'a [syn::NestedMetaItem];
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(attr) = self.attrs.next() {
            if let syn::MetaItem::List(ref ident, ref values) = attr.value {
                if ident == "conrod" {
                    return Some(&values[..]);
                }
            }
        }
        None
    }
}
