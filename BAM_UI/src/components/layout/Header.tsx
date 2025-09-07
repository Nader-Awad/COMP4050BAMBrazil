import React from "react";
import { motion } from "framer-motion";
import { Microscope, Users } from "lucide-react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";
import { Separator } from "../../components/ui/separator";

type Role = "student" | "teacher" | "admin";

type HeaderProps = {
  role: Role;
  setRole: (role: Role) => void;
  user: { name: string };
};

export default function Header({ role, setRole, user }: HeaderProps) {
  return (
    <header className="flex flex-col md:flex-row md:items-center md:justify-between gap-3">
      <div className="flex items-center gap-3">
        <motion.div initial={{ scale: 0.8, opacity: 0 }} animate={{ scale: 1, opacity: 1 }}>
          <div className="p-3 rounded-2xl bg-slate-900 text-white shadow-lg">
            <Microscope className="w-6 h-6" />
          </div>
        </motion.div>
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Bioscope Booking</h1>
          <p className="text-slate-500 text-sm">Reserve lab bioscopes, avoid conflicts, and keep things fair.</p>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <Select value={role} onValueChange={(v: Role) => setRole(v)}>
          <SelectTrigger className="w-[165px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="student">Student view</SelectItem>
            <SelectItem value="teacher">Teacher view</SelectItem>
            <SelectItem value="admin">Admin view</SelectItem>
          </SelectContent>
        </Select>
        <Separator orientation="vertical" className="h-8" />
        <div className="hidden md:flex items-center gap-2 text-sm text-slate-500">
          <Users className="w-4 h-4" />
          <span>{user.name}</span>
        </div>
      </div>
    </header>
  );
}

