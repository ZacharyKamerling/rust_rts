/*
if openlist.empty {
    None
}
else {
    let head = openlist.head();
    closedlist.add(head);

    for node in expand(head) {
        if !closedlist.contains(head) {
            openlist.add(node);
        }
    }
}
*/