import { Link } from "react-router-dom";
import { Construction } from "lucide-react";
import { Button } from "@/components/ui/Button";

interface Props {
  title: string;
  phase?: string;
}

export function NotImplemented({ title, phase = "later phase" }: Props) {
  return (
    <div className="mx-auto max-w-md px-4 py-16 text-center">
      <span className="mx-auto grid h-12 w-12 place-items-center rounded-full bg-amber-100 text-amber-600">
        <Construction size={24} />
      </span>
      <h1 className="mt-4 font-display text-2xl font-bold">{title}</h1>
      <p className="mt-2 text-sm text-gray-500">
        This screen is wired up in <strong>{phase}</strong>. The route, type definitions, and API
        contract are already in place — only the implementation is pending.
      </p>
      <Button variant="secondary" className="mt-6" as-child>
        <Link to="/">Back home</Link>
      </Button>
    </div>
  );
}
