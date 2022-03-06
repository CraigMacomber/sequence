/// Create an `enum` from a list of types implementing [crate::tree::Node] with members made from a list of types which implement [crate::tree::Node].
/// TODO:
/// 2. move logic from [indirect_nav] into here.
macro_rules! fromMembers {
    ( $(($name:ident, $chunk:ty)),+ $(,)?) => {
        #[derive(::derive_more::From)]
        pub enum Child<'a> {$(
            $name(<&'a $chunk as crate::chunk::Chunk>::Child),
        )*}

        #[derive(::derive_more::From)]
        pub enum TraitView<'a> {$(
            $name(<<&'a $chunk as crate::chunk::Chunk>::View as crate::tree::NodeNav<<&'a $chunk as crate::chunk::Chunk>::Child>>::TTraitChildren),
        )*}

        #[derive(Clone)]
        pub enum NodeView<'a> {$(
            $name(<&'a $chunk as crate::chunk::Chunk>::View),
        )*}

        impl<'a> crate::tree::NodeNav<Child<'a>> for NodeView<'a> {
            type TTraitChildren = TraitView<'a>;
            type TLabels = LabelIterator<'a>;

            fn get_traits(&self) -> Self::TLabels {
                match self {$(
                    NodeView::$name(n) => crate::tree::NodeNav::<<&'a $chunk as crate::chunk::Chunk>::Child>::get_traits(n).into(),
                )*}
            }

            fn get_trait(&self, label: crate::tree::Label) -> Self::TTraitChildren {
                match self {$(
                    NodeView::$name(n) => crate::tree::NodeNav::<<&'a $chunk as crate::chunk::Chunk>::Child>::get_trait(n, label).into(),
                )*}
            }
        }

        impl<'a> crate::tree::NodeData for NodeView<'a> {
            fn get_def(&self) -> crate::tree::Def {
                match self {$(
                    NodeView::$name(n) => crate::tree::NodeData::get_def(n),
                )*}
            }
            fn get_payload(&self) -> Option<crate::util::ImSlice>{
                match self {$(
                    NodeView::$name(n) => crate::tree::NodeData::get_payload(n),
                )*}
            }
        }

        impl<'a> crate::node_id::HasId for NodeView<'a> {
            fn get_id(&self) -> crate::node_id::NodeId {
                match self {$(
                    NodeView::$name(n) => crate::node_id::HasId::get_id(n),
                )*}
            }
        }

        impl<'a> Iterator for TraitView<'a> {
            type Item = Child<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                match self {$(
                    TraitView::$name(ref mut c) => c.next().map(|c| c.into()),
                )*}
            }
        }

        #[derive(::derive_more::From)]
        pub enum LabelIterator<'a> {$(
            $name(<<&'a $chunk as crate::chunk::Chunk>::View as crate::tree::NodeNav<<&'a $chunk as crate::chunk::Chunk>::Child>>::TLabels),
        )*}

        impl Iterator for LabelIterator<'_> {
            type Item = crate::tree::Label;

            fn next(&mut self) -> Option<Self::Item> {
                match self {$(
                    LabelIterator::$name(ref mut c) => c.next(),
                )*}
            }
        }

         /// Tree data, stored in the forest, keyed by the first id in the chunk.
         #[derive(Clone, PartialEq)]
        pub enum NavChunk {$(
            $name($chunk),
        )*}
    }
}

pub(crate) use fromMembers;
