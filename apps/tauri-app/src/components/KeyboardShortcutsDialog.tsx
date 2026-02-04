// Keyboard shortcuts dialog component
import { useUiStore } from "@/stores/uiStore";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/Dialog";
import { KEYBOARD_SHORTCUTS } from "@/hooks/useKeyboardShortcuts";

interface ShortcutSection {
  title: string;
  shortcuts: { keys: string[]; description: string }[];
}

const sections: ShortcutSection[] = [
  { title: "Global", shortcuts: KEYBOARD_SHORTCUTS.global },
  { title: "Tab Navigation", shortcuts: KEYBOARD_SHORTCUTS.tabs },
  { title: "Review Interface", shortcuts: KEYBOARD_SHORTCUTS.review },
  { title: "Task Detail", shortcuts: KEYBOARD_SHORTCUTS.taskDetail },
];

function ShortcutKey({ children }: { children: string }) {
  return (
    <kbd className="rounded border border-[hsl(var(--border))] bg-[hsl(var(--muted))] px-1.5 py-0.5 font-mono text-xs">
      {children}
    </kbd>
  );
}

function ShortcutRow({
  keys,
  description,
}: {
  keys: string[];
  description: string;
}) {
  return (
    <div className="flex items-center justify-between py-1.5">
      <span className="text-sm text-[hsl(var(--foreground))]">{description}</span>
      <div className="flex items-center gap-1">
        {keys.map((key, index) => (
          <span key={key} className="flex items-center gap-1">
            <ShortcutKey>{key}</ShortcutKey>
            {index < keys.length - 1 && (
              <span className="text-[hsl(var(--muted-foreground))]">+</span>
            )}
          </span>
        ))}
      </div>
    </div>
  );
}

function ShortcutSectionComponent({
  title,
  shortcuts,
}: ShortcutSection) {
  return (
    <div className="mb-4 last:mb-0">
      <h3 className="mb-2 text-sm font-medium text-[hsl(var(--muted-foreground))]">
        {title}
      </h3>
      <div className="space-y-0.5">
        {shortcuts.map((shortcut) => (
          <ShortcutRow
            key={shortcut.description}
            keys={shortcut.keys}
            description={shortcut.description}
          />
        ))}
      </div>
    </div>
  );
}

export function KeyboardShortcutsDialog() {
  const { isKeyboardShortcutsOpen, setKeyboardShortcutsOpen } = useUiStore();

  return (
    <Dialog open={isKeyboardShortcutsOpen} onOpenChange={setKeyboardShortcutsOpen}>
      <DialogContent className="max-w-md max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Keyboard Shortcuts</DialogTitle>
          <DialogDescription>
            Available keyboard shortcuts for navigating the application.
          </DialogDescription>
        </DialogHeader>
        <div className="mt-4">
          {sections.map((section) => (
            <ShortcutSectionComponent
              key={section.title}
              title={section.title}
              shortcuts={section.shortcuts}
            />
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}
