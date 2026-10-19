export function initials(name: string) {
  return name.split(/\s+/).slice(0, 2).map((part) => part[0]).join("").toLocaleUpperCase("es");
}

export function phoneHref(phone: string) {
  return `tel:${phone.replace(/\s/g, "")}`;
}
