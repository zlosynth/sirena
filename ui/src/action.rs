pub enum Action {
    AddNode(gazpatcho::model::Node),
    UpdateNode(gazpatcho::model::Node),
    RemoveNode(gazpatcho::model::Node),
    AddOutputPatch(gazpatcho::model::PinAddress),
    AddPatch(gazpatcho::model::Patch),
    RemovePatch(gazpatcho::model::Patch),
}
