
    pub fn initial_populate(&mut self) -> Result<(), Error> {
        self.config.properties.produced_by = self.create_property("produced by", &[])?;
        self.config.properties.instance_of = self.create_property("instance of", &[])?;
        self.config.properties.physical_mode = self.create_property("physical mode", &[])?;
        self.config.properties.gtfs_short_name = self.create_property("gtfs short name", &[])?;
        self.config.properties.gtfs_long_name = self.create_property("gtfs long name", &[])?;
        self.config.properties.gtfs_id = self.create_property("gtfs id", &[])?;
        Ok(())
    }
