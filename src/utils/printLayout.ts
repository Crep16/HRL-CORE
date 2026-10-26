import type { CompanySettings } from '../types/company';

export function printLayout(settings: CompanySettings) {
  const ticket=settings.paper!=='letter';
  const [width, height] = ticket ? [settings.paper==='ticket60' ? 60 : 80, 297] : settings.orientation === 'landscape' ? [279,216] : [216,279];
  const margin=ticket ? 3 : 8;
  const groups: number[] = [];
  const capacity = ticket ? 1 : 2;
  for (let remaining = settings.copies; remaining > 0; remaining -= capacity) groups.push(Math.min(capacity, remaining));
  return {width:width!, height:height!, margin, contentWidth:width!-margin*2, contentHeight:height!-margin*2, groups};
}
