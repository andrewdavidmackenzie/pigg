#[cfg(test)]
mod test {
    use pigdef::description::{PinDescription, PinDescriptionSet};
    use pigdef::pin_function::PinFunction;
    use std::borrow::Cow;

    #[test]
    fn get_hardware() {
        let hw = crate::get_hardware().expect("Could not get hardware");
        let description = hw.description("Test");
        let pins = description.pins.pins();
        assert_eq!(pins.len(), 40);
        assert_eq!(pins[0].name, "3V3")
    }

    #[test]
    fn hw_can_be_got() {
        let _hw = crate::get_hardware().expect("Could not get hardware");
    }

    #[test]
    fn forty_board_pins() {
        let hw = crate::get_hardware().expect("Could not get hardware");
        let pin_set = hw.description("Test").pins;
        assert_eq!(pin_set.pins().len(), 40);
    }

    #[test]
    fn bcm_pins_sort_in_order() {
        // 0-27, not counting the gpio0 and gpio1 pins with no options
        let hw = crate::get_hardware().expect("Could not get hardware");
        let pin_set = hw.description("Test").pins;
        let sorted_bcm_pins = pin_set.bcm_pins_sorted();
        assert_eq!(pin_set.bcm_pins_sorted().len(), 26);
        let mut previous = 1; // we start at GPIO2
        for pin in sorted_bcm_pins {
            assert_eq!(pin.bcm.expect("Could not get BCM pin number"), previous + 1);
            previous = pin.bcm.expect("Could not get BCM pin number");
        }
    }

    #[test]
    fn display_pin_description() {
        let pin = PinDescription {
            bpn: 7,
            bcm: Some(11),
            name: Cow::from("Fake Pin"),
            options: Cow::from(vec![]),
        };

        println!("Pin: {}", pin);
    }

    #[test]
    fn sort_bcm() {
        let pin7 = PinDescription {
            bpn: 7,
            bcm: Some(11),
            name: Cow::from("Fake Pin"),
            options: Cow::from(vec![PinFunction::Input(None), PinFunction::Output(None)]),
        };

        let pin8 = PinDescription {
            bpn: 8,
            bcm: Some(1),
            name: Cow::from("Fake Pin"),
            options: Cow::from(vec![PinFunction::Input(None), PinFunction::Output(None)]),
        };

        let pins = [
            pin7.clone(),
            pin8,
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
        ];
        let pin_set = PinDescriptionSet::new(&pins);
        assert_eq!(
            pin_set
                .pins()
                .first()
                .expect("Could not get pin")
                .bcm
                .expect("Could not get BCM Pin Number"),
            11
        );
        assert_eq!(
            pin_set
                .pins()
                .get(1)
                .expect("Could not get pin")
                .bcm
                .expect("Could not get BCM Pin Number"),
            1
        );
        assert_eq!(
            pin_set
                .bcm_pins_sorted()
                .first()
                .expect("Could not get pin")
                .bcm
                .expect("Could not get BCM Pin Number"),
            1
        );
        assert_eq!(
            pin_set
                .bcm_pins_sorted()
                .get(1)
                .expect("Could not get pin")
                .bcm
                .expect("Could not get BCM Pin Number"),
            11
        );
    }
}
