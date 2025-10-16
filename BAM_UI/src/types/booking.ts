export type Slot = { start: number; end: number };

export type Bioscope = { id: string; name: string };

//Bookings
export type Booking = {
  id: string;
  microscope_id: string;
  date: string;
  slot_start: number;
  slot_end: number;
  title: string;
  group_name: string | null;
  attendees: number | null;
  requester_id: string;
  requester_name: string;
  status: "Pending" | "Approved" | "Rejected";
  approved_by: string | null;
  created_at: string;
};

export type BookingDraft = {
  title: string;
  groupName: string;
  attendees: number;
  slot: string; // key: `${start}-${end}`
};

export type BookingFilters = {
  showApproved: boolean;
  showPending: boolean;
};

//Sessions
export type Session = {
  id: string;
  user_id: string;
  booking_id: string | null;
  microscope_id: string;
  status: "Active" | "Completed" | "Aborted";
  started_at: string;
  ended_at: string | null;
}

//Images
export interface BoundingBox {
  x: number;                  
  y: number;                  
  width: number;             
  height: number;             
}

export interface DetectedObject {
  class_name: string;
  confidence: number;       
  bounding_box: BoundingBox;
}

export type ImageMetadata = {
  objects_detected: DetectedObject[];
  focus_quality: number | null;           
  magnification: string | null;
  lighting_conditions: string | null;
}

export interface Image {
  id: string;                
  session_id: string;         
  filename: string;
  file_path: string;
  content_type: string;
  file_size: number;          
  width: number | null;       
  height: number | null;      
  metadata: ImageMetadata;
  captured_at: string;       
}

