import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { toast } from "sonner";
import { Loader2 } from "lucide-react";
import { useAuthStore } from "@/store/authStore";
import { Input } from "@/components/ui/Input";
import { Button } from "@/components/ui/Button";
import type { UserRole } from "@/types";

const schema = z.object({
  full_name: z.string().min(1, "Enter your full name"),
  email: z.string().email("Enter a valid email"),
  phone: z.string().min(7, "Enter a valid phone"),
  password: z.string().min(8, "At least 8 characters"),
  role: z.enum(["customer", "restaurant", "driver"]),
});
type FormValues = z.infer<typeof schema>;

const roleOptions: { value: UserRole; label: string; description: string }[] = [
  { value: "customer", label: "Customer", description: "Order food from restaurants" },
  { value: "restaurant", label: "Restaurant", description: "List your restaurant & receive orders" },
  { value: "driver", label: "Driver", description: "Deliver orders and earn" },
];

export function RegisterPage() {
  const navigate = useNavigate();
  const registerFn = useAuthStore((s) => s.register);
  const [submitting, setSubmitting] = useState(false);

  const {
    register,
    handleSubmit,
    watch,
    formState: { errors },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: { role: "customer" },
  });

  const selectedRole = watch("role");

  const onSubmit = async (values: FormValues) => {
    setSubmitting(true);
    try {
      await registerFn(values);
      toast.success("Account created!");
      navigate("/");
    } catch (e: unknown) {
      const msg =
        (e as { response?: { data?: { error?: { message?: string } } } })?.response?.data?.error
          ?.message ?? "Registration failed";
      toast.error(msg);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="mx-auto flex max-w-md flex-col gap-6 px-4 py-12">
      <div className="text-center">
        <h1 className="font-display text-3xl font-bold">Create your account</h1>
        <p className="mt-1 text-sm text-gray-500">Join Food City in 30 seconds</p>
      </div>

      <form onSubmit={handleSubmit(onSubmit)} className="card flex flex-col gap-4 p-6">
        <div>
          <span className="label">I am a…</span>
          <div className="grid grid-cols-3 gap-2">
            {roleOptions.map((opt) => (
              <button
                type="button"
                key={opt.value}
                onClick={() =>
                  (
                    document.getElementById("role-" + opt.value) as HTMLInputElement | null
                  )?.click()
                }
                className={`rounded-lg border p-3 text-left text-xs transition ${
                  selectedRole === opt.value
                    ? "border-brand-500 bg-brand-50 ring-2 ring-brand-100"
                    : "border-gray-200 hover:bg-gray-50"
                }`}
              >
                <div className="font-semibold">{opt.label}</div>
                <div className="mt-0.5 text-gray-500">{opt.description}</div>
              </button>
            ))}
          </div>
          {roleOptions.map((opt) => (
            <input
              key={opt.value}
              id={"role-" + opt.value}
              type="radio"
              value={opt.value}
              className="hidden"
              {...register("role")}
            />
          ))}
        </div>

        <Input label="Full name" placeholder="Jane Doe" error={errors.full_name?.message} {...register("full_name")} />
        <Input label="Email" type="email" placeholder="you@example.com" error={errors.email?.message} {...register("email")} />
        <Input label="Phone" placeholder="+1 555 123 4567" error={errors.phone?.message} {...register("phone")} />
        <Input label="Password" type="password" placeholder="••••••••" error={errors.password?.message} {...register("password")} />

        <Button type="submit" disabled={submitting} className="mt-2">
          {submitting ? <Loader2 size={16} className="animate-spin" /> : null}
          Create account
        </Button>
      </form>

      <p className="text-center text-sm text-gray-500">
        Already have an account?{" "}
        <Link to="/login" className="font-medium text-brand-600 hover:underline">
          Log in
        </Link>
      </p>
    </div>
  );
}
