import noteCss from '../assets/styles/company-note.css?inline';
import type { CompanySettings } from '../types/company';
import { printLayout } from './printLayout';
import { paginatePrintCopies } from './paginatePrintCopies';

export async function printCompanyNote(note: HTMLElement, settings: CompanySettings) {
  const iframe = document.createElement('iframe');
  iframe.title = 'Documento de impresión';
  iframe.style.cssText = 'position:fixed;width:1px;height:1px;left:-9999px;border:0';
  document.body.append(iframe);
  const doc = iframe.contentDocument;
  if (!doc || !iframe.contentWindow) { iframe.remove(); throw new Error('No se pudo preparar la impresión.'); }
  const layout=printLayout(settings);
  const size=`${layout.width}mm ${layout.height}mm`;
  doc.open();
  doc.write('<!doctype html><html lang="es"><head><meta charset="utf-8"><title>HRL · Nota de prueba</title></head><body></body></html>');
  doc.close();
  doc.title = note.querySelector('.note-heading')?.textContent?.trim() || 'HRL · Nota';
  const style = doc.createElement('style');
  style.textContent = `${noteCss}
    @page{size:${size};margin:${layout.margin}mm}
    body{margin:0;width:${layout.contentWidth}mm}
    .print-sheet{box-sizing:border-box;width:100%;height:${layout.contentHeight-.5}mm;display:grid;gap:8mm;position:relative;break-after:page}
    .print-sheet:last-child{break-after:auto}
    .print-stacked{grid-template-rows:repeat(var(--copy-count),minmax(0,1fr))}
    .print-side-by-side{grid-template-columns:repeat(var(--copy-count),minmax(0,1fr))}
    .print-copy{min-width:0;min-height:0}
    .print-pair::before{content:'';position:absolute;border-color:#7f909b;border-style:dashed;pointer-events:none}
    .print-stacked.print-pair::before{top:50%;width:100%;border-width:1px 0 0}
    .print-side-by-side.print-pair::before{left:50%;height:100%;border-width:0 0 0 1px}
    .print-sheet .company-note{padding:0;font:12px/1.4 Arial,sans-serif}
    .print-sheet .note-company{padding-bottom:6px;gap:8px}
    .print-sheet .note-company h3{font-size:16px;margin-bottom:3px}
    .print-sheet .note-company p{font-size:11px;margin:1px 0}
    .print-sheet .note-company img{width:70px;height:32px}
    .print-sheet .note-heading{margin-top:6px;gap:8px}
    .print-sheet .note-heading h4{font-size:13px}
    .print-sheet .note-date{margin:3px 0 6px;font-size:11px}
    .print-sheet .note-fields{gap:4px 10px;margin:6px 0}
    .print-sheet .note-fields dt{font-size:11px}
    .print-sheet .note-box{padding:5px 0}
    .print-sheet .note-box h5{font-size:12px;margin-bottom:2px}
    .print-sheet .note-clauses{margin-top:6px;padding-top:6px;font-size:11px;line-height:1.4}
    .print-sheet .note-clauses h5{font-size:12px}
    .print-sheet .note-clauses ol{margin:4px 0;padding-left:0;list-style:none}
    .print-sheet .note-clauses li{margin-bottom:3px}
    .print-sheet .note-clauses ol:not(.unnumbered) li::before{content:attr(data-print-number) '. '}
    .print-sheet .note-signature{margin:18px auto 6px;padding-top:3px}
    .print-sheet .note-footer{margin-top:6px;font-size:11px}
    .print-sheet .note-reverse{break-before:auto;margin:0;padding-top:6px}
    .print-sheet .note-totals{margin-block:6px}
    .print-sheet .print-continuation{font-size:11px;margin:4px 0}
    body>.print-copy .company-note{padding:0;width:100%;max-width:none;font:12px/1.45 Arial,sans-serif}
    body>.print-copy .note-company h3{font-size:16px}
    body>.print-copy .note-company p,body>.print-copy .note-date,body>.print-copy .note-fields dt{font-size:11px}
    body>.print-copy .note-clauses,body>.print-copy .note-footer{font-size:11px;line-height:1.45}
    body>.print-copy+.print-copy{break-before:page}
  `;
  doc.head.append(style);
  const source=doc.importNode(note,true);
  const staging=doc.createElement('div');staging.style.position='absolute';staging.style.visibility='hidden';staging.append(source);doc.body.append(staging);
  await Promise.all(Array.from(staging.querySelectorAll('img')).map(img => img.decode().catch(() => undefined)));
  try {
    if(settings.paper!=='letter') {
      for(let i=0;i<settings.copies;i++) {const copy=doc.createElement('div');copy.className='print-copy';copy.append(source.cloneNode(true));doc.body.append(copy);}
    } else paginatePrintCopies(doc,source,settings);
  } catch(error) {iframe.remove();throw error;}
  staging.remove();
  const frameWindow = iframe.contentWindow;
  frameWindow.addEventListener('afterprint', () => iframe.remove(), {once:true});
  try { frameWindow.focus(); frameWindow.print(); } catch { iframe.remove(); throw new Error('La impresión no está disponible en esta ventana.'); }
  // Some WebView hosts do not dispatch afterprint after closing the system dialog.
  setTimeout(() => iframe.remove(), 60_000);
}
