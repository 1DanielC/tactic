#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VendorType {
    Insta,
    Theta,
}

impl VendorType {
    pub fn vendor_id(&self) -> i32 {
        match self {
            VendorType::Insta => 1,
            VendorType::Theta => 2,
        }
    }

    pub fn from_vendor_id(vendor_id: i32) -> Option<Self> {
        match vendor_id {
            1 => Some(VendorType::Insta),
            2 => Some(VendorType::Theta),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Insta360OneX2,
    ThetaZ1,
}

impl DeviceType {
    pub fn vendor_type(&self) -> VendorType {
        match self {
            DeviceType::Insta360OneX2 => VendorType::Insta,
            DeviceType::ThetaZ1 => VendorType::Theta,
        }
    }

    pub fn product_id(&self) -> i32 {
        match self {
            DeviceType::Insta360OneX2 => 11,
            DeviceType::ThetaZ1 => 12,
        }
    }

    pub fn from_product_id(product_id: i32) -> Option<Self> {
        match product_id {
            11 => Some(DeviceType::Insta360OneX2),
            12 => Some(DeviceType::ThetaZ1),
            _ => None,
        }
    }
}
