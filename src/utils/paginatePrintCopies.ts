import type { CompanySettings } from '../types/company';
import { printLayout } from './printLayout';

// Split overflowing notes at DOM/text boundaries so both copies remain readable.
function textSlice(source: HTMLElement, start: number, end: number): HTMLElement {
  const doc=source.ownerDocument;
  const range=doc.createRange();
  range.selectNodeContents(source);
  const position=(offset:number) => {
    const walker=doc.createTreeWalker(source,4);
    let current:Node|null; let count=0;
    while((current=walker.nextNode())) {
      const length=current.textContent?.length || 0;
      if(count+length>=offset) return {node:current,offset:offset-count};
      count+=length;
    }
    return {node:source,offset:source.childNodes.length};
  };
  if(start>0) {const point=position(start);range.setStart(point.node,point.offset);}
  if(end<(source.textContent?.length || 0)) {const point=position(end);range.setEnd(point.node,point.offset);}
  const copy=source.cloneNode(false) as HTMLElement;
  copy.append(range.cloneContents());
  return copy;
}

export function paginatePrintCopies(doc: Document, source: HTMLElement, settings: CompanySettings) {
  const layout=printLayout(settings);
  const landscape=settings.orientation==='landscape';
  const front=source.querySelector<HTMLElement>('.note-front');
  if(!front) throw new Error('No se encontró el contenido de la nota.');
  source.querySelectorAll('.note-clauses ol:not(.unnumbered)').forEach(list=>{
    Array.from(list.children).forEach((item,index)=>(item as HTMLElement).dataset.printNumber=String(index+1));
  });
  const headers=Array.from(front.children).filter(child=>child.matches('.note-company,.note-heading,.note-date'));
  const parts=[Array.from(front.children).filter(child=>!headers.includes(child))];
  const reverse=source.querySelector<HTMLElement>('.note-reverse');
  if(reverse) parts.push([reverse]);

  for(const count of layout.groups) {
    for(const [partIndex,part] of parts.entries()) {
      const queue=part.map(node=>node.cloneNode(true) as HTMLElement);
      let pageIndex=0;
      while(queue.length) {
        const sheet=doc.createElement('section');
        sheet.className=`print-sheet ${count===2 ? 'print-pair' : ''} ${landscape ? 'print-side-by-side' : 'print-stacked'}`;
        sheet.style.setProperty('--copy-count',String(count));
        doc.body.append(sheet);
        const slot=doc.createElement('div'); slot.className='print-copy'; sheet.append(slot);
        const article=source.cloneNode(false) as HTMLElement; slot.append(article);
        const content=doc.createElement('div');content.className='note-front';article.append(content);
        for(const header of headers) content.append(header.cloneNode(true));
        if(partIndex>0 || pageIndex>0) {
          const label=doc.createElement('p');label.className='print-continuation';
          label.textContent=[partIndex>0 ? 'Reverso' : '',pageIndex>0 ? 'Continuación' : ''].filter(Boolean).join(' · ');
          content.append(label);
        }
        const fits=()=>article.getBoundingClientRect().height<=slot.getBoundingClientRect().height+.25;
        let added=0;
        while(queue.length) {
          const node=queue.shift()!;content.append(node);
          if(fits()) {added++;continue;}
          node.remove();
          if(added) {queue.unshift(node);break;}
          // A single long paragraph may be taller than an entire copy area.
          const text=node.textContent || '';
          let low=0,high=text.length;
          while(low<high) {
            const mid=Math.ceil((low+high)/2);
            const candidate=textSlice(node,0,mid);content.append(candidate);
            const fit=fits();candidate.remove();
            if(fit) low=mid; else high=mid-1;
          }
          if(!low) throw new Error('El encabezado no deja espacio para el contenido. Reduce sus datos o elige una hoja más grande.');
          const wordBreak=text.lastIndexOf(' ',low-1)+1;
          const cut=wordBreak>low/2 ? wordBreak : low;
          content.append(textSlice(node,0,cut));
          if(cut<text.length) queue.unshift(textSlice(node,cut,text.length));
          added++;break;
        }
        for(let copy=1;copy<count;copy++) sheet.append(slot.cloneNode(true));
        pageIndex++;
      }
    }
  }
}
