use crate::types::*;


struct Wall {
    rect: Rect,
}

pub struct Mobile {
    pub rect: Rect,
    pub vx: i32,
    pub vy: i32,
}

#[allow(dead_code)]
impl Mobile {
    pub fn new(rect: Rect, vx: i32, vy: i32) -> Self{
        Self {
            rect, vx, vy
        }
    }
    pub fn update(&mut self) {
        self.rect.translate(self.vx, self.vy)  
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum ColliderID {
    Static(usize),
    Dynamic(usize),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Contact {
    a: ColliderID,
    b: ColliderID,
    mtv: (i32, i32),
}

#[allow(dead_code)]
fn rect_touching(r1: Rect, r2: Rect) -> bool {
    // r1 left is left of r2 right
    r1.x <= r2.x+r2.w as i32 &&
        // r2 left is left of r1 right
        r2.x <= r1.x+r1.w as i32 &&
        // those two conditions handle the x axis overlap;
        // the next two do the same for the y axis:
        r1.y <= r2.y+r2.h as i32 &&
        r2.y <= r1.y+r1.h as i32
}

#[allow(dead_code)]
fn rect_displacement(r1: Rect, r2: Rect) -> Option<(i32, i32)> {
    // Draw this out on paper to double check, but these quantities
    // will both be positive exactly when the conditions in rect_touching are true.
    let x_overlap = (r1.x + r1.w as i32).min(r2.x + r2.w as i32) - r1.x.max(r2.x);
    let y_overlap = (r1.y + r1.h as i32).min(r2.y + r2.h as i32) - r1.y.max(r2.y);
    if x_overlap >= 0 && y_overlap >= 0 {
        // This will return the magnitude of overlap in each axis.
        Some((x_overlap, y_overlap))
    } else {
        None
    }
}

// Here we will be using push() on into, so it can't be a slice
#[allow(dead_code)]
fn gather_contacts(statics: &[Wall], dynamics: &[Mobile], into: &mut Vec<Contact>) {
    // collide mobiles against mobiles
    for (ai, a) in dynamics.iter().enumerate() {
        for (bi, b) in dynamics.iter().enumerate().skip(ai + 1) {
            if rect_touching(a.rect, b.rect) {
                let overlap = rect_displacement(a.rect, b.rect).unwrap();
                let mut mtv = (0, 0);
                if overlap.0 > overlap.1 {
                    mtv.1 = overlap.1;
                } else {
                    mtv.0 = overlap.0;
                }
                into.push(Contact {
                    a: ColliderID::Dynamic(ai),
                    b: ColliderID::Dynamic(bi),
                    mtv,
                });
            }
        }
    }
    // collide mobiles against walls
    for (ai, a) in dynamics.iter().enumerate() {
        for (bi, b) in statics.iter().enumerate() {
            if rect_touching(a.rect, b.rect) {
                let overlap = rect_displacement(a.rect, b.rect).unwrap();
                let mut mtv = (0, 0);
                if overlap.0 > overlap.1 {
                    mtv.1 = overlap.1;
                } else {
                    mtv.0 = overlap.0;
                }
                into.push(Contact {
                    a: ColliderID::Dynamic(ai),
                    b: ColliderID::Static(bi),
                    mtv,
                });
            }
        }
    }
}

#[allow(dead_code)]
fn restitute(statics: &[Wall], dynamics: &mut [Mobile], contacts: &mut [Contact]) {
    // handle restitution of dynamics against dynamics and dynamics against statics wrt contacts.
    // You could instead make contacts `Vec<Contact>` if you think you might remove contacts.
    // You could also add an additional parameter, a slice or vec representing how far we've displaced each dynamic, to avoid allocations if you track a vec of how far things have been moved.
    // You might also want to pass in another &mut Vec<Contact> to be filled in with "real" touches that actually happened.
    contacts.sort_unstable_by_key(|c| -(c.mtv.0 * c.mtv.0 + c.mtv.1 * c.mtv.1));
    for contact in contacts {
        //just make a move cause its dynamic for sure
        match contact.a {
            ColliderID::Static(_) => {
                println!("uhh");

            }
            ColliderID::Dynamic(index_a) => match contact.b {
                ColliderID::Dynamic(_) => {
                    println!("GAME OVER");

                    //put in a trigger to call this in main to kill program or bring to new page
                    //*control_flow = ControlFlow::Exit;
                    return;
                }
                ColliderID::Static(index_b) => {
                    let obj_a = &mut dynamics[index_a];
                    let obj_b = &statics[index_b];
                    let a_under = obj_a.rect.y > obj_b.rect.y;
                    let a_totheright = obj_a.rect.x > obj_b.rect.x;
                    let mut x_translate = contact.mtv.0;
                    let mut y_translate = contact.mtv.1;
                    if x_translate != 0 {
                        obj_a.vx = 0;
                    }
                    if y_translate != 0 {
                        obj_a.vy = 0;
                    }
                    if !a_under {
                        y_translate *= -1;
                    }
                    if !a_totheright {
                        x_translate *= -1;
                    }
                    obj_a.rect.translate(x_translate, y_translate);
                    // println!("{} by {}", x_translate, y_translate);
                }
            },
        }
    }
    // Keep going!  Note that you can assume every contact has a dynamic object in .a.
    // You might decide to tweak the interface of this function to separately take dynamic-static and dynamic-dynamic contacts, to avoid a branch inside of the response calculation.
    // Or, you might decide to calculate signed mtvs taking direction into account instead of the unsigned displacements from rect_displacement up above.  Or calculate one MTV per involved entity, then apply displacements to both objects during restitution (sorting by the max or the sum of their magnitudes)
}