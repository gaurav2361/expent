"use client";

import { Avatar, AvatarFallback, AvatarImage } from "@expent/ui/components/avatar";
import { Button } from "@expent/ui/components/button";
import { toast } from "@expent/ui/components/goey-toaster";
import { Input } from "@expent/ui/components/input";
import { Label } from "@expent/ui/components/label";
import { Separator } from "@expent/ui/components/separator";
import { CameraIcon, LoaderIcon } from "lucide-react";
import * as React from "react";
import { apiClient } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";

export default function SettingsProfilePage() {
  const session = useSession();
  const user = session.data?.user;

  const [avatarPreview, setAvatarPreview] = React.useState<string | null>(null);
  const [isUploading, setIsUploading] = React.useState(false);
  const [name, setName] = React.useState("");
  const [username, setUsername] = React.useState("");
  const [email, setEmail] = React.useState("");
  const [isSaving, setIsSaving] = React.useState(false);

  // Populate form fields when session loads
  React.useEffect(() => {
    if (user) {
      setName(user.name || "");
      setUsername((user as unknown as Record<string, string>).username || "");
      setEmail(user.email || "");
      if (user.image) {
        setAvatarPreview(user.image);
      }
    }
  }, [user]);

  const handleAvatarChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    // Validate file type
    if (!file.type.startsWith("image/")) {
      toast.error("Please select an image file.");
      return;
    }

    // Validate file size (max 5MB, will be compressed server-side)
    if (file.size > 5 * 1024 * 1024) {
      toast.error("Image must be under 5MB.");
      return;
    }

    // Show local preview immediately
    const reader = new FileReader();
    reader.onloadend = () => {
      setAvatarPreview(reader.result as string);
    };
    reader.readAsDataURL(file);

    // Upload to R2 via the avatar endpoint
    setIsUploading(true);
    try {
      const formData = new FormData();
      formData.append("file", file);

      const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:7878";
      const response = await fetch(`${API_BASE_URL}/api/users/avatar`, {
        method: "POST",
        body: formData,
        credentials: "include",
      });

      if (!response.ok) {
        const errorText = await response.text().catch(() => "Upload failed");
        throw new Error(errorText);
      }

      const result = (await response.json()) as { url: string; key: string };
      setAvatarPreview(result.url);

      // Refetch session so the new avatar URL propagates to NavUser & everywhere else
      await session.refetch();

      toast.success("Avatar updated successfully!");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to upload avatar");
      // Revert preview to session avatar on failure
      if (user?.image) {
        setAvatarPreview(user.image);
      } else {
        setAvatarPreview(null);
      }
    } finally {
      setIsUploading(false);
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await apiClient("/api/users/profile", {
        method: "PUT",
        body: JSON.stringify({
          name: name || undefined,
          username: username || undefined,
        }),
      });
      await session.refetch();
      toast.success("Profile updated successfully!");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to update profile");
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="space-y-6 w-full max-w-2xl">
      <div>
        <h3 className="text-lg font-medium">Profile</h3>
        <p className="text-sm text-muted-foreground">This is how others will see you on the site.</p>
      </div>
      <Separator />
      <div className="space-y-8">
        {/* Profile Picture */}
        <div className="space-y-2">
          <Label>Profile Picture</Label>
          <div className="flex items-center gap-6">
            <div className="relative group">
              <Avatar className="h-20 w-20">
                <AvatarImage src={avatarPreview || undefined} alt="Profile picture" />
                <AvatarFallback className="text-2xl bg-primary/10 text-primary">
                  {user?.name?.charAt(0) ?? "U"}
                </AvatarFallback>
              </Avatar>
              <label
                htmlFor="avatar-upload"
                className="absolute inset-0 flex items-center justify-center rounded-full bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer"
              >
                {isUploading ? (
                  <LoaderIcon className="h-5 w-5 text-white animate-spin" />
                ) : (
                  <CameraIcon className="h-5 w-5 text-white" />
                )}
              </label>
              <input
                id="avatar-upload"
                type="file"
                accept="image/*"
                className="sr-only"
                onChange={handleAvatarChange}
                disabled={isUploading}
              />
            </div>
            <div className="space-y-1">
              <p className="text-sm font-medium">Upload a new avatar</p>
              <p className="text-[0.8rem] text-muted-foreground">JPG, PNG or GIF. Max 5MB. Auto-compressed to WebP.</p>
              <label
                htmlFor="avatar-upload"
                className="inline-flex items-center justify-center rounded-md text-sm font-medium border border-input bg-background hover:bg-accent hover:text-accent-foreground h-8 px-3 cursor-pointer transition-colors"
              >
                {isUploading ? "Uploading…" : "Choose File"}
              </label>
            </div>
          </div>
        </div>

        {/* Name */}
        <div className="space-y-2">
          <Label htmlFor="name">Name</Label>
          <Input id="name" value={name} onChange={(e) => setName(e.target.value)} placeholder="Your name" />
        </div>

        {/* Username */}
        <div className="space-y-2">
          <Label htmlFor="username">Username</Label>
          <Input
            id="username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            placeholder="expent_user"
          />
          <p className="text-[0.8rem] text-muted-foreground">
            This is your public display name. It can be your real name or a pseudonym.
          </p>
        </div>

        {/* Email */}
        <div className="space-y-2">
          <Label htmlFor="email">Email</Label>
          <Input id="email" type="email" value={email} disabled className="opacity-60" />
          <p className="text-[0.8rem] text-muted-foreground">
            Email cannot be changed from this page. Contact support to update your email.
          </p>
        </div>

        <Button onClick={handleSave} disabled={isSaving}>
          {isSaving ? "Saving…" : "Update profile"}
        </Button>
      </div>
    </div>
  );
}
