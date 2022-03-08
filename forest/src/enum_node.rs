/// Create an `enum` from a list of types implementing [crate::tree::Node] with members made from a list of types which implement [crate::tree::Node].
macro_rules! fromMembers {
    (
    $(#[$enum_meta:meta])*
    $pub:vis
    enum $Enum:ident {
        $(
            $name:ident ($chunk:ty)
        ),* $(,)?
    }
) => {
        pub mod $Enum {
            use super::*;

            /// Tree data, stored in the forest, keyed by the first id in the chunk.
            #[derive(Clone, PartialEq, ::derive_more::From)]
            pub enum Chunk {$(
                $name($chunk),
            )*}

            #[derive(::derive_more::From)]
            pub enum Child<'a> {$(
                $name(<&'a $chunk as crate::chunk::Chunk>::Child),
            )*}

            #[derive(::derive_more::From)]
            pub enum TraitView<'a> {$(
                $name(<<&'a $chunk as crate::chunk::Chunk>::View as crate::tree::NodeNav<<&'a $chunk as crate::chunk::Chunk>::Child>>::TTraitChildren),
            )*}

            #[derive(Clone)]
            pub enum Node<'a> {$(
                $name(<&'a $chunk as crate::chunk::Chunk>::View),
            )*}

            impl<'a> crate::tree::NodeNav<Child<'a>> for Node<'a> {
                type TTraitChildren = TraitView<'a>;
                type TLabels = LabelIterator<'a>;

                fn get_traits(&self) -> Self::TLabels {
                    match self {$(
                        Node::$name(n) => crate::tree::NodeNav::<<&'a $chunk as crate::chunk::Chunk>::Child>::get_traits(n).into(),
                    )*}
                }

                fn get_trait(&self, label: crate::tree::Label) -> Self::TTraitChildren {
                    match self {$(
                        Node::$name(n) => crate::tree::NodeNav::<<&'a $chunk as crate::chunk::Chunk>::Child>::get_trait(n, label).into(),
                    )*}
                }
            }

            impl<'a> crate::tree::NodeData for Node<'a> {
                fn get_def(&self) -> crate::tree::Def {
                    match self {$(
                        Node::$name(n) => crate::tree::NodeData::get_def(n),
                    )*}
                }
                fn get_payload(&self) -> Option<crate::util::ImSlice>{
                    match self {$(
                        Node::$name(n) => crate::tree::NodeData::get_payload(n),
                    )*}
                }
            }

            impl<'a> crate::node_id::HasId for Node<'a> {
                fn get_id(&self) -> crate::node_id::NodeId {
                    match self {$(
                        Node::$name(n) => crate::node_id::HasId::get_id(n),
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

            #[derive(::derive_more::From)]
            pub enum Expander<'a>
            {$(
                $name(<&'a $chunk as crate::chunk::Chunk>::Expander),
            )*}

            impl<'a> Iterator for Expander<'a> {
                type Item = Node<'a>;

                fn next(&mut self) -> Option<Self::Item> {
                    match self {
                        Expander::Uniform(ref mut c) => c.next().map(Node::Uniform),
                        Expander::Indirect(ref mut c) => c.next().map(Node::Indirect),
                    }
                }
            }

            impl<'a> crate::chunk::Chunk for &'a Chunk {
                type View = Node<'a>;
                type Child = Child<'a>;
                type Expander = Expander<'a>;
                fn get(&self, first_id: crate::node_id::NodeId, id: crate::node_id::NodeId) -> Option<Node<'a>> {
                    match self {$(
                        Chunk::$name(node) => node.get(first_id, id).map(Node::$name),
                    )*}
                }

                fn top_level_nodes(&self, id: crate::node_id::NodeId) -> Self::Expander {
                    match self {$(
                        Chunk::$name(c) => c.top_level_nodes(id).into(),
                    )*}
                }
            }

            /// For parent info: Allow viewing the tree of chunks as Node.
            impl<'a> crate::tree::NodeNav<crate::chunk::ChunkId> for &'a Chunk {
                type TTraitChildren = ChunkTraitIterator<'a>;
                type TLabels = ChunkLabelIterator<'a>;

                fn get_traits(&self) -> Self::TLabels {
                    match self {$(
                        Chunk::$name(s) => s.get_traits().into(),
                    )*}
                }

                fn get_trait(&self, label: crate::tree::Label) -> Self::TTraitChildren {
                    match self {$(
                        Chunk::$name(s) => s.get_trait(label).into(),
                    )*}
                }
            }

            #[derive(::derive_more::From)]
            pub enum ChunkLabelIterator<'a> {$(
                $name(<&'a $chunk as crate::tree::NodeNav<crate::chunk::ChunkId>>::TLabels),
            )*}

            #[derive(::derive_more::From)]
            pub enum ChunkTraitIterator<'a> {$(
                $name(<&'a $chunk as crate::tree::NodeNav<crate::chunk::ChunkId>>::TTraitChildren),
            )*}

            impl<'a> Iterator for ChunkLabelIterator<'a> {
                type Item = crate::tree::Label;

                fn next(&mut self) -> Option<Self::Item> {
                    match self {$(
                        ChunkLabelIterator::$name(i) => i.next(),
                    )*}
                }
            }

            impl<'a> Iterator for ChunkTraitIterator<'a> {
                type Item = crate::chunk::ChunkId;

                fn next(&mut self) -> Option<Self::Item> {
                    match self {$(
                        ChunkTraitIterator::$name(i) => i.next(),
                    )*}
                }
            }
        }
    }
}

pub(crate) use fromMembers;
