use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone, Default)]
pub struct EntitiesId {
    pub properties: Properties,
    pub items: Items,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Properties {
    /// The Topo ID is used as an internal ID based on the name of each known properties/items
    /// It makes it possible to query the DB to get all the known entities
    pub topo_id_id: String,
    /// Link to the data Producer
    pub produced_by: String,
    /// Type of the entity
    pub instance_of: String,
    /// Basic name in the GTFS
    pub gtfs_name: String,
    /// Short name in the GTFS
    pub gtfs_short_name: String,
    /// Long name in the GTFS
    pub gtfs_long_name: String,
    /// ID in the GTFS
    pub gtfs_id: String,
    /// Link to the Datasource
    pub data_source: String,
    pub first_seen_in: String,
    /// Path of the file used to import the data
    pub source: String,
    /// Format of the file used to import the data
    pub file_format: String,
    /// sha256 of the file used to import the data
    pub sha_256: String,
    /// Link to the Physical mode of the entity
    pub has_physical_mode: String,
    /// version of the tool used to import the entities
    pub tool_version: String,
    /// Shows a relation of inclusion: a stop point is part_of a stop area
    pub part_of: String,
    /// Shows that a stop is connected to a line https://www.wikidata.org/wiki/Property:P81
    pub connecting_line: String,
    /// The coordinate of an entity
    pub coordinate_location: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Items {
    pub physical_mode: String,
    pub route: String,
    pub producer: String,
    pub tramway: String,
    pub subway: String,
    pub railway: String,
    pub bus: String,
    pub ferry: String,
    pub cable_car: String,
    pub gondola: String,
    pub funicular: String,
    pub stop_point: String,
    pub stop_area: String,
    pub stop_entrance: String,
    pub stop_boarding_area: String,
    pub stop_generic_node: String,
}

impl EntitiesId {
    pub fn physical_mode(&self, route: &gtfs_structures::Route) -> &str {
        use gtfs_structures::RouteType::*;
        match route.route_type {
            Tramway => &self.items.tramway,
            Subway => &self.items.subway,
            Rail => &self.items.railway,
            Bus => &self.items.bus,
            Ferry => &self.items.ferry,
            CableCar => &self.items.cable_car,
            Gondola => &self.items.gondola,
            Funicular => &self.items.funicular,
            _ => &self.items.bus,
        }
    }

    pub fn location_type(&self, stop: &gtfs_structures::Stop) -> &str {
        use gtfs_structures::LocationType::*;
        match stop.location_type {
            StopPoint => &self.items.stop_point,
            StopArea => &self.items.stop_area,
            StationEntrance => &self.items.stop_entrance,
            GenericNode => &self.items.stop_generic_node,
            BoardingArea => &self.items.stop_boarding_area,
        }
    }
}
