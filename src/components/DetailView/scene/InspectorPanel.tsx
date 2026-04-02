import { PreviewInspectorSection } from '../../../types/clipboard';
import { Card, CardContent, CardHeader, CardTitle } from '../../ui/card';

interface InspectorPanelProps {
  sections: PreviewInspectorSection[];
}

export function InspectorPanel({ sections }: InspectorPanelProps) {
  if (sections.length === 0) {
    return null;
  }

  return (
    <div className="space-y-2">
      {sections.map((section) => (
        <Card
          key={section.title}
          className="overflow-hidden rounded-[16px] border-border/70 bg-card/95 shadow-[0_8px_20px_rgba(15,23,42,0.04)]"
        >
          <CardHeader className="px-3 pb-1.5 pt-2.5">
            <CardTitle className="text-xs font-semibold uppercase tracking-[0.12em] text-muted-foreground">
              {section.title}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-1.5 px-3 pb-3">
            {section.items.map((item) => (
              <div
                key={`${section.title}-${item.label}`}
                className="grid grid-cols-[minmax(0,auto)_minmax(0,1fr)] items-start gap-2 rounded-[14px] border border-border/60 bg-muted/35 px-2 py-1.5"
              >
                <span className="text-[11px] leading-4 text-muted-foreground">{item.label}</span>
                <span
                  className={
                    item.mono
                      ? 'break-all text-right font-mono text-[11px] leading-4'
                      : 'break-words text-right text-[11px] leading-4'
                  }
                >
                  {item.value}
                </span>
              </div>
            ))}
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
