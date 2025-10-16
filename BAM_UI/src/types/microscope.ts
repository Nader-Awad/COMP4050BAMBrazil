export type CommandType =
  | "Move"
  | "Focus"
  | "Capture"
  | "SetMagnification"
  | "SetLighting"
  | "StartTracking"
  | "StopTracking";

export type MicroscopeCommand = {
  command_type: CommandType;
  parameters: unknown;
}

export type Position = {
  x: number;   
  y: number;    
  z: number;    
}

export type FocusInfo = {
  is_focused: boolean;
  focus_score: number | null;      
  auto_focus_active: boolean;
}

export type LightingInfo = {
  intensity: number;                
  color_temperature: number | null; 
}


export type MicroscopeStatus = {
  microscope_id: string;
  is_connected: boolean;
  current_session: string | null;   
  position: Position;
  focus: FocusInfo;
  magnification: string;
  lighting: LightingInfo;
  tracking_active: boolean;
}
