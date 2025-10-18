import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Card, CardContent } from "@components/ui/card";
import { Input } from "@components/ui/input";
import { Label } from "@components/ui/label";
import { Button } from "@components/ui/button";
import { useAuthContext } from "@context/auth-context";

export default function LoginPage() {
  const { login } = useAuthContext();
  const nav = useNavigate();

  const [email, setEmail] = useState("");
  const [pw, setPw] = useState("");
  const [showPw, setShowPw] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const validEmail = /\S+@\S+\.\S+/.test(email);
  const validPw = pw.length >= 8;

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!validEmail) return setError("Please enter a valid email address.");
    if (!validPw) return setError("Password must be at least 8 characters.");
    setError(null);
    setSubmitting(true);
    try {
      await login(email, pw);
      nav("/post-login");
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Login failed. Check your credentials and try again.";
      setError(msg);
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="min-h-screen bg-gradient-to-b from-slate-50 to-white py-12 px-4">
      <div className="mx-auto w-full max-w-md">
        <div className="mb-8 text-center">
          <h1 className="text-2xl font-semibold text-slate-800">Sign in to Bioscope</h1>
          <p className="mt-1 text-sm text-slate-500">Use your school email and password</p>
        </div>

        <Card className="border-0 shadow-sm rounded-2xl">
          <CardContent className="p-6 md:p-8">
            <form onSubmit={onSubmit} className="space-y-5">
              <div className="space-y-2">
                <Label htmlFor="email" className="text-slate-700">Email</Label>
                <Input
                  id="email"
                  type="email"
                  placeholder="user@bam.edu"
                  value={email}
                  onChange={(e) => { setEmail(e.target.value); if (error) setError(null); }}
                  className="h-11"
                  autoComplete="email"
                  required
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="password" className="text-slate-700">Password</Label>
                <div className="relative">
                  <Input
                    id="password"
                    type={showPw ? "text" : "password"}
                    placeholder="••••••••"
                    value={pw}
                    onChange={(e) => { setPw(e.target.value); if (error) setError(null); }}
                    className="h-11 pr-20"
                    autoComplete="current-password"
                    required
                    minLength={8}
                  />
                  <button
                    type="button"
                    onClick={() => setShowPw((s) => !s)}
                    className="absolute right-2 top-1/2 -translate-y-1/2 text-sm text-slate-500 hover:text-slate-700 px-2 py-1 rounded-md"
                    aria-label={showPw ? "Hide password" : "Show password"}
                  >
                    {showPw ? "Hide" : "Show"}
                  </button>
                </div>
              </div>

              {error && (
                <div className="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700" role="alert" aria-live="polite">
                  {error}
                </div>
              )}

              <Button type="submit" disabled={submitting || !validEmail || !validPw} className="w-full h-11 rounded-xl">
                {submitting ? "Signing in…" : "Sign in"}
              </Button>
            </form>
          </CardContent>
        </Card>

        <p className="mt-6 text-center text-xs text-slate-500">
          By continuing, you agree to our acceptable use and lab safety policies.
        </p>
      </div>
    </div>
  );
}
