import { useAuthStore } from "@/store/authStore";
import { useAddresses, useCreateAddress, useDeleteAddress } from "@/hooks/useAddresses";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { User, MapPin, Plus, Trash2, LogOut } from "lucide-react";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";

export function ProfilePage() {
  const { user, logout } = useAuthStore();
  const navigate = useNavigate();
  const { data: addresses, isLoading: addrLoading } = useAddresses();
  const createAddr = useCreateAddress();
  const deleteAddr = useDeleteAddress();
  const [showForm, setShowForm] = useState(false);
  const [newAddr, setNewAddr] = useState({
    label: "Home",
    line1: "",
    city: "Colombo",
    lat: 6.9271,
    lng: 79.8612,
  });

  if (!user) return null;

  const handleAdd = async () => {
    if (!newAddr.line1.trim()) {
      toast.error("Address line is required");
      return;
    }
    await createAddr.mutateAsync({
      ...newAddr,
      formatted_address: `${newAddr.line1}, ${newAddr.city}`,
    });
    setNewAddr({ ...newAddr, line1: "" });
    setShowForm(false);
  };

  return (
    <div className="mx-auto max-w-2xl px-4 py-8">
      <h1 className="font-display text-2xl font-bold mb-6">Profile</h1>

      {/* Account info */}
      <section className="card p-5 mb-4">
        <h2 className="font-semibold flex items-center gap-2 mb-3">
          <User size={16} /> Account
        </h2>
        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-gray-500">Name</span>
            <span className="font-medium">{user.full_name}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-500">Email</span>
            <span className="font-medium">{user.email}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-500">Phone</span>
            <span className="font-medium">{user.phone}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-500">Role</span>
            <span className="font-medium capitalize">{user.role}</span>
          </div>
        </div>
        <Button
          variant="ghost"
          size="sm"
          className="mt-4 text-red-600"
          onClick={() => {
            logout();
            navigate("/");
          }}
        >
          <LogOut size={14} /> Log out
        </Button>
      </section>

      {/* Addresses */}
      <section className="card p-5">
        <div className="flex items-center justify-between mb-3">
          <h2 className="font-semibold flex items-center gap-2">
            <MapPin size={16} /> Saved addresses
          </h2>
          <Button variant="ghost" size="sm" onClick={() => setShowForm((s) => !s)}>
            <Plus size={14} /> Add
          </Button>
        </div>

        {showForm && (
          <div className="mb-3 space-y-2 border border-gray-100 rounded-lg p-3">
            <Input
              label="Label"
              value={newAddr.label}
              onChange={(e) => setNewAddr((s) => ({ ...s, label: e.target.value }))}
            />
            <Input
              label="Street address"
              placeholder="123 Main St"
              value={newAddr.line1}
              onChange={(e) => setNewAddr((s) => ({ ...s, line1: e.target.value }))}
            />
            <Input
              label="City"
              value={newAddr.city}
              onChange={(e) => setNewAddr((s) => ({ ...s, city: e.target.value }))}
            />
            <Button size="sm" onClick={handleAdd} disabled={createAddr.isPending}>
              Save address
            </Button>
          </div>
        )}

        {addrLoading ? (
          <p className="text-sm text-gray-500">Loading…</p>
        ) : addresses && addresses.length > 0 ? (
          <div className="space-y-2">
            {addresses.map((a) => (
              <div
                key={a.id}
                className="flex items-start justify-between gap-2 rounded-lg border border-gray-100 p-3"
              >
                <div>
                  <div className="font-medium text-sm">
                    {a.label}
                    {a.is_default && (
                      <span className="ml-2 text-xs text-gray-500">(default)</span>
                    )}
                  </div>
                  <div className="text-sm text-gray-600">
                    {a.line1}
                    {a.line2 ? `, ${a.line2}` : ""}, {a.city}
                  </div>
                </div>
                <button
                  onClick={() => {
                    if (confirm("Delete this address?")) deleteAddr.mutate(a.id);
                  }}
                  className="text-red-500 hover:bg-red-50 rounded p-1"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-sm text-gray-500">No saved addresses yet.</p>
        )}
      </section>
    </div>
  );
}
